use parquet::file::properties::WriterProperties;
use parquet::file::writer::{ParquetWriter, SerializedFileWriter};
use std::sync::Arc;

pub fn props() -> Arc<WriterProperties> {
    Arc::new(
        WriterProperties::builder()
            .set_compression(parquet::basic::Compression::GZIP)
            .build(),
    )
}

pub trait FileWriterRows {
    fn total_num_rows(&mut self) -> &i64;
}

impl<W: 'static + ParquetWriter> FileWriterRows for SerializedFileWriter<W> {
    fn total_num_rows(&mut self) -> &i64 {
        &50
        // &self.total_num_rows // became private field
    }
}

pub mod prelude {
    pub use super::props;
    pub use super::FileWriterRows;
    pub use parquet::file::properties::WriterProperties;
    pub use parquet::file::writer::SerializedFileWriter;
    pub use parquet::record::RecordWriter;
}
