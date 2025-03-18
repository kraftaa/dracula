use core::pin::Pin;
use core::task::{Context, Poll};
use futures::{Future, Stream};
use pin_project::pin_project;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct FileStream {
    buf_reader: BufReader<tokio::fs::File>,
}

impl FileStream {
    pub async fn _new(path: &str) -> Self {
        let file = tokio::fs::File::open(path).await.unwrap();
        FileStream {
            buf_reader: BufReader::new(file),
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut buf = vec![0; 1024];
        let mut next = Box::pin(self.buf_reader.read(&mut buf[..]));
        match next.as_mut().poll(ctx) {
            Poll::Ready(Ok(bytes_read)) => {
                if bytes_read == 0 {
                    Poll::Ready(None)
                } else {
                    buf.resize(bytes_read, 0);
                    Poll::Ready(Some(Ok(buf)))
                }
            }
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::stream::StreamExt;
    use std::io::Write;

    #[tokio::test]
    async fn blah() {
        let mut file = std::fs::File::create("/tmp/blah.txt").unwrap();
        let buf = vec![10; 1024];
        file.write_all(&buf[..]).unwrap();
        file.write_all(&[15; 100]).unwrap();

        let mut file_stream = FileStream::_new("/tmp/blah.txt").await;

        let mut reads = vec![];
        while let Some(Ok(buf)) = file_stream.next().await {
            reads.push(buf.len())
        }

        assert_eq!(reads, vec![1024, 100]);
    }
}
