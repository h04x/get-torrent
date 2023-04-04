/*
 * source code from bendy example
*/
use bendy::decoding::Decoder;
use bendy::{
    decoding::{Error, FromBencode, Object, ResultExt},
    encoding::AsString,
};

use sha1::{Digest, Sha1};

#[derive(Debug)]
pub struct MetaInfo {
    pub announce: String,
    pub info: Info,
    pub info_hash: [u8; 20],
    pub comment: Option<String>,         // not official element
    pub creation_date: Option<u64>,      // not official element
    pub http_seeds: Option<Vec<String>>, // not official element
}

#[derive(Debug)]
pub struct Info {
    pub piece_length: String,
    pub pieces: Vec<u8>,
    pub name: String,
    pub file_length: Option<String>,
}

impl AsRef<[u8]> for Info {
    fn as_ref(&self) -> &[u8] {
        self.piece_length.as_bytes()
    }
}

impl FromBencode for MetaInfo {
    const EXPECTED_RECURSION_DEPTH: usize = Info::EXPECTED_RECURSION_DEPTH + 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut announce = None;
        let mut comment = None;
        let mut creation_date = None;
        let mut http_seeds = None;
        let mut info = None;
        let mut info_hash = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"announce", value) => {
                    announce = String::decode_bencode_object(value)
                        .context("announce")
                        .map(Some)?;
                }
                (b"comment", value) => {
                    comment = String::decode_bencode_object(value)
                        .context("comment")
                        .map(Some)?;
                }
                (b"creation date", value) => {
                    creation_date = u64::decode_bencode_object(value)
                        .context("creation_date")
                        .map(Some)?;
                }
                (b"httpseeds", value) => {
                    http_seeds = Vec::decode_bencode_object(value)
                        .context("http_seeds")
                        .map(Some)?;
                }
                (b"info", value) => {
                    if let Object::Dict(x) = value {
                        let raw = x.into_raw().unwrap();
                        let mut decoder = Decoder::new(raw);
                        let mut hasher = Sha1::new();
                        hasher.update(raw);
                        info_hash = Some(hasher.finalize().into());
                        //println!("{:x}", result);
                        info = Info::decode_bencode_object(Object::Dict(
                            decoder
                                .next_object()
                                .unwrap()
                                .unwrap()
                                .try_into_dictionary()
                                .unwrap(),
                        ))
                        .context("info")
                        .map(Some)?;
                    }
                    /*info = Info::decode_bencode_object(value)
                    .context("info")
                    .map(Some)?;*/
                    //println!("{:?}", value.try_into_bytes().unwrap());
                }
                (unknown_field, _) => {
                    /*return Err(Error::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));*/
                }
            }
        }

        let announce = announce.ok_or_else(|| Error::missing_field("announce"))?;
        let info = info.ok_or_else(|| Error::missing_field("info"))?;
        let info_hash = info_hash.ok_or_else(|| Error::missing_field("info"))?;

        Ok(MetaInfo {
            announce,
            info,
            info_hash,
            comment,
            creation_date,
            http_seeds,
        })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 5;

    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut file_length = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"length", value) => {
                    file_length = value
                        .try_into_integer()
                        .context("file.length")
                        .map(ToString::to_string)
                        .map(Some)?;
                }
                (b"name", value) => {
                    name = String::decode_bencode_object(value)
                        .context("name")
                        .map(Some)?;
                }
                (b"piece length", value) => {
                    piece_length = value
                        .try_into_integer()
                        .context("length")
                        .map(ToString::to_string)
                        .map(Some)?;
                }
                (b"pieces", value) => {
                    pieces = AsString::decode_bencode_object(value)
                        .context("pieces")
                        .map(|bytes| Some(bytes.0))?;
                }
                (unknown_field, _) => {
                    /*return Err(Error::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));*/
                }
            }
        }

        //let file_length = file_length.ok_or_else(|| Error::missing_field("file_length"))?;
        let name = name.ok_or_else(|| Error::missing_field("name"))?;
        let piece_length = piece_length.ok_or_else(|| Error::missing_field("piece_length"))?;
        let pieces = pieces.ok_or_else(|| Error::missing_field("pieces"))?;

        // Check that we discovered all necessary fields
        Ok(Info {
            file_length,
            name,
            piece_length,
            pieces,
        })
    }
}
