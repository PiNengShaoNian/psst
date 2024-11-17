use std::{
    fs::File,
    io::{self, Cursor, Read},
    sync::Arc,
};

use druid::im::HashMap;

use crate::data::Config;

const MAGIC_BYTES: &[u8] = b"SPCO";
const FILE_TYPE: &[u8] = b"LocalFilesStorage";

const ARRAY_SIGNATURE: u8 = 0x60;
const STRING_SIGNATURE: u8 = 0x09;
const TRAILER_END: [u8; 2] = [0x78u8, 0x04u8];

#[derive(Clone, Debug)]
pub struct LocalTrack {
    title: Arc<str>,
    path: Arc<str>,
    album: Arc<str>,
    artist: Arc<str>,
}

pub struct LocalTrackManager {
    tracks: HashMap<Arc<str>, Vec<LocalTrack>>,
}

impl LocalTrackManager {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
        }
    }

    pub fn load_tracks_for_user(&mut self, username: &str) -> io::Result<()> {
        let file_path =
            Config::spotify_local_files_file(username).ok_or(io::ErrorKind::NotFound)?;
        let local_file = File::open(&file_path)?;
        let mut reader = LocalTracksReader::new(local_file)?;

        log::info!("parsing local tracks: {:?}", file_path);

        // Start reading the track array.
        let num_tracks = reader.read_array()?;
        if num_tracks > 0 {
            reader.advance(2)?; // Skip `0x94 0x00`.
        }

        self.tracks.clear();

        for n in 1..=num_tracks {
            let title = reader.read_string()?;
            let artist = reader.read_string()?;
            let album = reader.read_string()?;
            let path = reader.read_string()?;
            let track = LocalTrack {
                title: title.into(),
                path: path.into(),
                album: album.into(),
                artist: artist.into(),
            };
            self.tracks
                .entry(track.title.clone())
                .or_default()
                .push(track);
            if reader.advance_until(&TRAILER_END).is_err() {
                if n != num_tracks {
                    log::warn!("found EOF but missing {} tracks", num_tracks - n);
                }
                break;
            }
        }

        Ok(())
    }
}

struct LocalTracksReader {
    chunked: ChunkedReader,
}

impl LocalTracksReader {
    fn new(file: File) -> io::Result<Self> {
        Ok(Self {
            chunked: Self::parse_file(file)?,
        })
    }

    /// Checks if `file` is in correct format and prepares it for reading.
    fn parse_file(mut file: File) -> io::Result<ChunkedReader> {
        // Validate the magic.
        let magic = read_bytes(&mut file, 4)?;
        if magic != MAGIC_BYTES {
            return Err(io::ErrorKind::InvalidData.into());
        }
        // Skip `0x13, 0x00*4`.
        advance(&mut file, 5)?;
        // Validate the file-type marker.
        let file_type = read_bytes(&mut file, 18)?;
        if file_type[0] != FILE_TYPE.len() as u8 || &file_type[1..] != FILE_TYPE {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(ChunkedReader::new(file))
    }

    fn read_array(&mut self) -> io::Result<usize> {
        let signature = read_u8(&mut self.chunked)?;
        if signature != ARRAY_SIGNATURE {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let num_entries = read_uvarint(&mut self.chunked)? as usize;
        Ok(num_entries)
    }

    fn advance(&mut self, len: usize) -> io::Result<()> {
        advance(&mut self.chunked, len)
    }

    fn advance_until(&mut self, bytes: &[u8]) -> io::Result<()> {
        advance_until(&mut self.chunked, bytes)
    }

    fn read_string(&mut self) -> io::Result<String> {
        let signature = read_u8(&mut self.chunked)?;
        if signature != STRING_SIGNATURE {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let str_size = read_uvarint(&mut self.chunked)?;
        let str_buf = read_utf8(&mut self.chunked, str_size as usize)?;
        Ok(str_buf)
    }
}

/// Implements a `Read` trait over the chunked file format described above.
struct ChunkedReader {
    inner: File,
    chunk: Cursor<Vec<u8>>,
}

impl ChunkedReader {
    fn new(inner: File) -> Self {
        Self {
            inner,
            chunk: Cursor::default(),
        }
    }

    fn read_next_chunk(&mut self) -> io::Result<()> {
        // Two LE bytes of chunk length.
        let size = read_u16_le(&mut self.inner)?;
        // Chunk content.
        let buf = read_bytes(&mut self.inner, size as usize)?;
        self.chunk = Cursor::new(buf);
        Ok(())
    }
}

impl Read for ChunkedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let n = self.chunk.read(buf)?;
            if n > 0 {
                break Ok(n);
            } else {
                // `self.chunk` is empty, read the next one.  Returns `Err` on EOF.
                self.read_next_chunk()?;
            }
        }
    }
}

/// Helper, reads a byte from `f` or returns `Err`.
fn read_u8(f: &mut impl io::Read) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    f.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Helper, reads little-endian `u16` or returns `Err`.
fn read_u16_le(f: &mut impl io::Read) -> io::Result<u16> {
    let mut buf = [0u8, 2];
    f.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Helper, reads ProtoBuf-style unsigned varint from `f` or returns `Err`.
fn read_uvarint(f: &mut impl io::Read) -> io::Result<u64> {
    let mut shift: u64 = 0;
    let mut ret: u64 = 0;

    loop {
        let byte = read_u8(f)?;
        let has_msb: bool = (byte & !0b01111111) != 0;
        ret |= ((byte & 0b01111111) as u64) << shift;

        if has_msb {
            shift += 7;
        } else {
            break;
        }
    }

    Ok(ret)
}

/// Helper, reads a `Vec<u8>` of length `len` from `f` or returns `Err`.
fn read_bytes(f: &mut impl io::Read, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

/// Helper, reads a UTF-8 string of length `len` from `f` or returns `Err`.
fn read_utf8(f: &mut impl io::Read, len: usize) -> io::Result<String> {
    let buf = read_bytes(f, len)?;
    String::from_utf8(buf).map_err(|_| io::ErrorKind::InvalidData.into())
}

/// Helper, skips `len` bytes of `f` or returns `Err`.
fn advance(f: &mut impl io::Read, len: usize) -> io::Result<()> {
    for _ in 0..len {
        read_u8(f)?;
    }
    Ok(())
}

/// Helper, skips bytes of `f` until an exact continuous `bytes` match is found,
/// or returns `Err`.
pub fn advance_until(f: &mut impl io::Read, bytes: &[u8]) -> io::Result<()> {
    let mut i = 0;
    while i < bytes.len() {
        loop {
            let r = read_u8(f)?;
            if r == bytes[i] {
                i += 1; // Match, continue with the next byte of `bytes`.
                break;
            } else {
                i = 0; // Mismatch, start at the beginning again.
                if r == bytes[i] {
                    i += 1;
                }
            }
        }
    }
    Ok(())
}
