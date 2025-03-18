pub mod dracula_parquet;
pub use self::dracula_parquet::*;

pub mod prelude {
    pub use parquet;
    pub use parquet::file::reader::{FileReader, SerializedFileReader};
    pub use parquet::file::writer::{FileWriter, SerializedFileWriter};
    pub use parquet::record::RecordWriter;
    pub use parquet_derive::*;
}
