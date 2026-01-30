use {
    chrono::NaiveDate,
    std::path::{Path, PathBuf},
    tokio::{
        io::{AsyncWrite, AsyncWriteExt, BufWriter},
        task::JoinHandle,
    },
};

pub type FlushTask = JoinHandle<std::io::Result<()>>;

pub trait FileHandler {
    type File: AsyncWrite + Send + Unpin + 'static;

    async fn create_file(&mut self, path: PathBuf) -> anyhow::Result<Self::File>;

    async fn create_date_stamped_file(
        &mut self,
        dir: &Path,
        prefix: &str,
        date: NaiveDate,
    ) -> anyhow::Result<BufWriter<Self::File>> {
        let date = date.format("%Y%m%d");
        let filename = format!("{prefix}-{date}");
        let file = self.create_file(dir.join(filename)).await?;
        Ok(BufWriter::new(file))
    }

    async fn close_file(
        &mut self,
        mut output: BufWriter<Self::File>,
        dir: &Path,
        prefix: &str,
        date: NaiveDate,
    ) -> anyhow::Result<(BufWriter<Self::File>, FlushTask)> {
        let new_output = self.create_date_stamped_file(dir, prefix, date).await?;
        let flush_task = tokio::spawn(async move { output.flush().await });
        Ok((new_output, flush_task))
    }
}

pub struct TokioFileHandler;

impl FileHandler for TokioFileHandler {
    type File = tokio::fs::File;

    async fn create_file(&mut self, path: PathBuf) -> anyhow::Result<Self::File> {
        let file = tokio::fs::File::create(path).await?;
        Ok(file)
    }
}

#[cfg(test)]
pub mod in_mem {
    use {
        super::*,
        pin_project_lite::pin_project,
        std::{
            collections::HashMap,
            sync::{Arc, RwLock},
        },
    };

    pin_project! {
        pub struct InMemFile {
            destination: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
            path: PathBuf,
        }
    }

    impl AsyncWrite for InMemFile {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let projection = self.project();
            let mut map = projection.destination.write().unwrap();
            let inner = map.get_mut(projection.path).unwrap();
            let pinned = std::pin::pin!(inner);
            pinned.poll_write(cx, buf)
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let projection = self.project();
            let mut map = projection.destination.write().unwrap();
            let inner = map.get_mut(projection.path).unwrap();
            let pinned = std::pin::pin!(inner);
            pinned.poll_flush(cx)
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let projection = self.project();
            let mut map = projection.destination.write().unwrap();
            let inner = map.get_mut(projection.path).unwrap();
            let pinned = std::pin::pin!(inner);
            pinned.poll_shutdown(cx)
        }

        fn poll_write_vectored(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let projection = self.project();
            let mut map = projection.destination.write().unwrap();
            let inner = map.get_mut(projection.path).unwrap();
            let pinned = std::pin::pin!(inner);
            pinned.poll_write_vectored(cx, bufs)
        }

        fn is_write_vectored(&self) -> bool {
            let map = self.destination.read().unwrap();
            let inner = map.get(&self.path).unwrap();
            inner.is_write_vectored()
        }
    }

    #[derive(Default)]
    pub struct InMemFileHandler {
        files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    }

    impl InMemFileHandler {
        pub fn into_inner(self) -> HashMap<PathBuf, Vec<u8>> {
            let lock = Arc::into_inner(self.files).unwrap();
            lock.into_inner().unwrap()
        }
    }

    impl FileHandler for InMemFileHandler {
        type File = InMemFile;

        async fn create_file(&mut self, path: PathBuf) -> anyhow::Result<Self::File> {
            let mut map = self.files.write().unwrap();
            map.insert(path.clone(), Vec::new());

            let file = InMemFile {
                destination: self.files.clone(),
                path,
            };
            Ok(file)
        }
    }
}
