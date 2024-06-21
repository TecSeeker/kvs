use crate::error::KvError;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    collections::HashMap,
    fmt,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};

const MAX_LOG_SIZE: u64 = 1024 * 1024; // The max log size, 1MB.

pub type Result<T> = std::result::Result<T, KvError>;

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Set { key, value } => write!(f, "Set {{ key: {}, value: {} }}", key, value),
            Command::Remove { key } => write!(f, "Remove {{ key: {} }}", key),
        }
    }
}

pub struct KvStore {
    index: HashMap<String, u64>,
    path: PathBuf,
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

impl KvStore {
    /// Open the KvStore from the given path
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let log_path = path.join("kvs.log");
        let log = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&log_path)?;
        let reader = BufReader::new(log.try_clone()?);
        let writer = BufWriter::new(log);
        let mut store = KvStore {
            index: HashMap::new(),
            path,
            reader,
            writer,
        };
        store.load()?;
        Ok(store)
    }

    /// Load the KvStore from the log file and build the in-memory index
    fn load(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<Command>();
        while let Some(cmd) = stream.next() {
            let cmd = cmd?;
            let pos = stream.byte_offset() as u64 - serde_json::to_string(&cmd).unwrap().len() as u64;
            match cmd {
                Command::Set { key, .. } => {
                    self.index.insert(key, pos);
                }
                Command::Remove { key } => {
                    self.index.remove(&key);
                }
            }
        }
        Ok(())
    }

    /// Set the value for a given key
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.writer.seek(SeekFrom::End(0))?;
        let cmd = Command::Set {
            key: key.clone(),
            value,
        };
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        self.index.insert(key, pos);
        self.check_compact()?;
        Ok(())
    }

    /// Get the value for a given key
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(&pos) = self.index.get(&key) {
            self.reader.seek(SeekFrom::Start(pos))?;
            let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<Command>();

            if let Some(cmd) = stream.next() {
                let cmd = cmd?;

                if let Command::Set { key: _, value } = cmd {
                    return Ok(Some(value));
                }
            }
        }
        Ok(None)
    }

    /// Remove a given key
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.remove(&key).is_none() {
            return Err(KvError::KeyNotFound);
        }
        let cmd = Command::Remove { key };
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        self.check_compact()?;
        Ok(())
    }

    /// Check if the log file needs to be compacted
    fn check_compact(&mut self) -> Result<()> {
        let log_path = self.path.join("kvs.log");
        let log_metadata = std::fs::metadata(&log_path)?;
        if log_metadata.len() > MAX_LOG_SIZE {
            self.compact()?;
        }
        Ok(())
    }

    /// Compact the log file
    fn compact(&mut self) -> Result<()> {
        let temp_path = self.path.join("temp_kvs.log");
        let mut temp_writer = BufWriter::new(File::create(&temp_path)?);

        for (key, &pos) in &self.index {
            self.reader.seek(SeekFrom::Start(pos))?;
            if let Some(cmd) = Deserializer::from_reader(&mut self.reader)
                .into_iter::<Command>()
                .next()
            {
                let cmd = cmd?;
                if let Command::Set { key: _, value } = cmd {
                    let new_cmd = Command::Set {
                        key: key.clone(),
                        value,
                    };
                    serde_json::to_writer(&mut temp_writer, &new_cmd)?;
                    temp_writer.flush()?;
                }
            }
        }

        std::fs::rename(temp_path, self.path.join("kvs.log"))?;

        let log = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(self.path.join("kvs.log"))?;
        self.writer = BufWriter::new(log.try_clone()?);
        self.reader = BufReader::new(log);

        Ok(())
    }
}

/// Function to print the content of BufReader for debugging purposes
#[allow(dead_code)]
fn print_reader_content(reader: &mut BufReader<File>) -> Result<()> {
    let current_pos = reader.stream_position()?;

    reader.seek(SeekFrom::Start(0))?;
    let mut temp_reader = reader.get_ref().try_clone().map(BufReader::new)?;

    let mut stream = Deserializer::from_reader(&mut temp_reader).into_iter::<Command>();
    let mut pos = 0;
    while let Some(cmd) = stream.next() {
        let cmd = cmd?;
        println!("Offset: {}, Command: {}", pos, cmd);
        pos = stream.byte_offset() as u64;
    }

    reader.seek(SeekFrom::Start(current_pos))?;
    Ok(())
}
