use {
    crate::clock::Clock,
    chrono::{DateTime, NaiveDate, TimeDelta, Utc},
    std::mem,
    tokio::io::{AsyncBufRead, AsyncBufReadExt},
};

pub struct Reader<C: Clock, R> {
    input: R,
    clock: C,
    date: DateTime<Utc>,
    instant: C::Instant,
    buf: Vec<u8>,
}

pub enum ReaderOutput {
    Eof,
    Line(NaiveDate),
}

impl<C: Clock, R: AsyncBufRead + Unpin> Reader<C, R> {
    pub fn new(mut clock: C, input: R) -> Self {
        let instant = clock.now();
        Self {
            input,
            clock,
            date: Utc::now(),
            instant,
            buf: Vec::with_capacity(1024),
        }
    }

    pub async fn read_line(&mut self, read_buf: &mut Vec<u8>) -> anyhow::Result<ReaderOutput> {
        self.buf.clear();
        let n_bytes = self.input.read_until(b'\n', &mut self.buf).await?;

        if n_bytes == 0 {
            return Ok(ReaderOutput::Eof);
        }

        let duration = self.clock.elapsed(&self.instant);
        self.instant += duration;
        let delta = TimeDelta::from_std(duration)?;
        self.date += delta;

        mem::swap(&mut self.buf, read_buf);
        Ok(ReaderOutput::Line(self.date.date_naive()))
    }
}
