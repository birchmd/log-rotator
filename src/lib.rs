use {
    self::{config::Config, reader::ReaderOutput},
    chrono::Utc,
    std::{collections::VecDeque, mem},
    tokio::io::{AsyncBufRead, AsyncWriteExt},
};

pub mod config;
pub mod reader;

mod clock;
mod file_handler;

#[cfg(test)]
mod test;

pub async fn log_redirect<R>(input: R, config: &Config) -> anyhow::Result<()>
where
    R: AsyncBufRead + Unpin,
{
    log_redirect_generic(
        input,
        config,
        clock::StdClock,
        &mut file_handler::TokioFileHandler,
    )
    .await
}

async fn log_redirect_generic<C, H, R>(
    input: R,
    config: &Config,
    clock: C,
    handler: &mut H,
) -> anyhow::Result<()>
where
    C: clock::Clock,
    H: file_handler::FileHandler,
    R: AsyncBufRead + Unpin,
{
    let mut flush_queue = VecDeque::new();
    let mut reader = reader::Reader::new(clock, input);
    let mut read_buf = Vec::with_capacity(1024);

    let mut date = Utc::now().date_naive();
    let mut output = handler
        .create_date_stamped_file(&config.dir, &config.prefix, date)
        .await?;
    let mut write_buf = Vec::with_capacity(1024);

    let mut reader_output = reader.read_line(&mut read_buf).await;

    while let Ok(ReaderOutput::Line(read_date)) = reader_output {
        if read_date != date {
            let (new_output, flush_task) = handler
                .close_file(output, &config.dir, &config.prefix, read_date)
                .await?;
            output = new_output;
            flush_queue.push_back(flush_task);
            date = read_date;
        }

        loop {
            match flush_queue.front() {
                Some(handle) if handle.is_finished() => {
                    // We intentionally ignore the errors from flushing because it's
                    // kind of too late to do anything about it anyway.
                    resolve_flush_task(&mut flush_queue).await.ok();
                }
                _ => break,
            }
        }

        mem::swap(&mut read_buf, &mut write_buf);
        let write_task = output.write_all(&write_buf);
        let read_task = reader.read_line(&mut read_buf);

        let (write_outcome, read_outcome) = tokio::join!(write_task, read_task);
        write_outcome?;
        reader_output = read_outcome;
    }

    if let Err(e) = reader_output {
        output
            .write_all(format!("LOGS ROTATION READER ERROR {e:?}").as_bytes())
            .await?;
    }

    // Close out the last file, then wait for all flushes to finish
    let tomorrow = date
        .succ_opt()
        .ok_or_else(|| anyhow::Error::msg("Tomorrow will never come."))?;
    let (_, flush_task) = handler
        .close_file(output, &config.dir, &config.prefix, tomorrow)
        .await?;
    flush_queue.push_back(flush_task);
    while flush_queue.front().is_some() {
        resolve_flush_task(&mut flush_queue).await.ok();
    }

    Ok(())
}

async fn resolve_flush_task(queue: &mut VecDeque<file_handler::FlushTask>) -> anyhow::Result<()> {
    if let Some(handle) = queue.pop_front() {
        handle.await??
    }
    Ok(())
}
