//! Compression gzip, zlib, deflate brut et Brotli, sur des octets quelconques.
//!
//! La décompression s'arrête au plafond [`MAX_OUTPUT_BYTES`] : une « bombe » de quelques
//! kilo-octets qui se déploierait en gigaoctets échoue sans remplir la mémoire.

use crate::codec::MAX_OUTPUT_BYTES;
use crate::conversion::megabytes;
use crate::options::{OptionKind, OptionSpec};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};
use flate2::read::{DeflateDecoder, MultiGzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use std::io::{self, Read, Write};

/// Niveau de gzip, zlib et deflate ; 6 est celui de l'outil `gzip`.
const LEVEL: OptionSpec = OptionSpec {
    id: "level",
    label: "Niveau",
    kind: OptionKind::Int {
        default: 6,
        min: 0,
        max: 9,
    },
};

/// Qualité de Brotli ; 11 est celle de l'outil `brotli`.
const QUALITY: OptionSpec = OptionSpec {
    id: "level",
    label: "Niveau",
    kind: OptionKind::Int {
        default: 11,
        min: 0,
        max: 11,
    },
};

/// Fenêtre de Brotli : 2^22 octets, sa valeur par défaut.
const BROTLI_WINDOW: u32 = 22;
const BROTLI_BUFFER: usize = 4096;

static GZIP_META: CodecMeta = CodecMeta {
    id: "gzip",
    label: "gzip",
    icon: "gz",
    category: Category::Compression,
    aliases: &["gz", "gunzip"],
    reversible: true,
    encodes_text: false,
    options: &[LEVEL],
};

static ZLIB_META: CodecMeta = CodecMeta {
    id: "zlib",
    label: "zlib",
    icon: "zl",
    category: Category::Compression,
    aliases: &[],
    reversible: true,
    encodes_text: false,
    options: &[LEVEL],
};

static DEFLATE_META: CodecMeta = CodecMeta {
    id: "deflate",
    label: "Deflate (brut)",
    icon: "df",
    category: Category::Compression,
    aliases: &["inflate", "raw_deflate"],
    reversible: true,
    encodes_text: false,
    options: &[LEVEL],
};

static BROTLI_META: CodecMeta = CodecMeta {
    id: "brotli",
    label: "Brotli",
    icon: "br",
    category: Category::Compression,
    aliases: &["br"],
    reversible: true,
    encodes_text: false,
    options: &[QUALITY],
};

pub struct Gzip;

impl Codec for Gzip {
    fn meta(&self) -> &'static CodecMeta {
        &GZIP_META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut encoder = GzEncoder::new(Vec::new(), level(options, &LEVEL)?);
        encoder.write_all(input).map_err(write_failed)?;
        encoder.finish().map_err(write_failed)
    }

    /// Plusieurs membres gzip à la suite sont décompressés l'un après l'autre, comme `gunzip`.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        if !input.starts_with(&[0x1f, 0x8b]) {
            return Err(wrong_header(
                "gzip",
                "un flux gzip commence par les octets 1f 8b",
            ));
        }
        inflate(MultiGzDecoder::new(input), "gzip", MAX_OUTPUT_BYTES)
    }
}

pub struct Zlib;

impl Codec for Zlib {
    fn meta(&self) -> &'static CodecMeta {
        &ZLIB_META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), level(options, &LEVEL)?);
        encoder.write_all(input).map_err(write_failed)?;
        encoder.finish().map_err(write_failed)
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        // En-tête de 2 octets : méthode 8 (deflate) et somme de contrôle multiple de 31 (RFC 1950).
        let valid_header = match input {
            [cmf, flg, ..] => {
                cmf & 0x0f == 8 && (u16::from(*cmf) << 8 | u16::from(*flg)).is_multiple_of(31)
            }
            _ => false,
        };
        if !valid_header {
            return Err(wrong_header(
                "zlib",
                "un flux zlib commence le plus souvent par 78",
            ));
        }
        inflate(ZlibDecoder::new(input), "zlib", MAX_OUTPUT_BYTES)
    }
}

/// Deflate sans en-tête (RFC 1951) : celui de la liaison SAML HTTP-Redirect et des fichiers ZIP.
pub struct Deflate;

impl Codec for Deflate {
    fn meta(&self) -> &'static CodecMeta {
        &DEFLATE_META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut encoder = DeflateEncoder::new(Vec::new(), level(options, &LEVEL)?);
        encoder.write_all(input).map_err(write_failed)?;
        encoder.finish().map_err(write_failed)
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        inflate(DeflateDecoder::new(input), "deflate", MAX_OUTPUT_BYTES)
    }
}

pub struct Brotli;

impl Codec for Brotli {
    fn meta(&self) -> &'static CodecMeta {
        &BROTLI_META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let quality = u32::try_from(options.int(&QUALITY)?).unwrap_or(11);
        let mut writer =
            brotli::CompressorWriter::new(Vec::new(), BROTLI_BUFFER, quality, BROTLI_WINDOW);
        writer.write_all(input).map_err(write_failed)?;
        // `into_inner` termine le flux avant de rendre le tampon.
        Ok(writer.into_inner())
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        inflate(
            brotli::Decompressor::new(input, BROTLI_BUFFER),
            "Brotli",
            MAX_OUTPUT_BYTES,
        )
    }
}

fn level(options: &Options, spec: &OptionSpec) -> Result<flate2::Compression, CodecError> {
    let level = u32::try_from(options.int(spec)?).unwrap_or(6);
    Ok(flate2::Compression::new(level))
}

/// Décompresse au plus `limit` octets ; au-delà, la lecture s'arrête et la conversion échoue.
fn inflate(reader: impl Read, format: &str, limit: usize) -> Result<Vec<u8>, CodecError> {
    let mut output = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut output)
        .map_err(|error| corrupt(format, &error))?;
    if output.len() > limit {
        return Err(CodecError::new(
            ErrorCode::OutputTooLarge,
            format!(
                "Décompression {format} arrêtée à {} : la sortie dépasse le plafond par conversion (bombe de décompression ?).",
                megabytes(limit)
            ),
        ));
    }
    Ok(output)
}

fn corrupt(format: &str, error: &io::Error) -> CodecError {
    let message = if error.kind() == io::ErrorKind::UnexpectedEof {
        format!("Flux {format} tronqué : les données s'arrêtent avant la fin du flux.")
    } else {
        format!("Flux {format} invalide : données corrompues ou compressées dans un autre format.")
    };
    CodecError::new(ErrorCode::InvalidCompressedData, message)
}

fn wrong_header(format: &str, hint: &str) -> CodecError {
    CodecError::new(
        ErrorCode::InvalidCompressedData,
        format!("Ce ne sont pas des données {format} : {hint}. Faut-il d'abord décoder du Base64 ou de l'hexadécimal ?"),
    )
}

/// Écrire dans un `Vec` ne peut pas échouer : ce serait une erreur interne.
fn write_failed(error: io::Error) -> CodecError {
    CodecError::new(
        ErrorCode::Internal,
        format!("Compression impossible : {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &[u8] = b"Hello, Glass Converter! Hello, Glass Converter!";

    fn all() -> [&'static dyn Codec; 4] {
        [&Gzip, &Zlib, &Deflate, &Brotli]
    }

    #[test]
    fn every_level_roundtrips() {
        for codec in all() {
            let max = match codec.meta().options[0].kind {
                OptionKind::Int { max, .. } => max,
                _ => unreachable!(),
            };
            for level in [0, 1, max] {
                let options = Options::new().with("level", level);
                let compressed = codec.encode(HELLO, &options).unwrap();
                let restored = codec.decode(&compressed, &Options::new()).unwrap();
                assert_eq!(restored, HELLO, "{} niveau {level}", codec.meta().id);
            }
        }
    }

    // Vecteurs produits par une autre implémentation : zlib de Node.js
    // (`gzipSync`, `deflateSync`, `deflateRawSync` et `brotliCompressSync` de « Hello »).
    #[test]
    fn decodes_streams_from_other_tools() {
        let vectors: [(&dyn Codec, &str); 4] = [
            (&Gzip, "1f8b080000000000000af348cdc9c907008289d1f705000000"),
            (&Zlib, "789cf348cdc9c90700058c01f5"),
            (&Deflate, "f348cdc9c90700"),
            (&Brotli, "0b028048656c6c6f03"),
        ];
        for (codec, hex) in vectors {
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            assert_eq!(
                codec.decode(&bytes, &Options::new()).unwrap(),
                b"Hello",
                "{}",
                codec.meta().id
            );
        }
    }

    #[test]
    fn concatenated_gzip_members_are_all_decoded() {
        let mut stream = Gzip.encode(b"Hello, ", &Options::new()).unwrap();
        stream.extend(Gzip.encode(b"world", &Options::new()).unwrap());
        assert_eq!(
            Gzip.decode(&stream, &Options::new()).unwrap(),
            b"Hello, world"
        );
    }

    #[test]
    fn wrong_or_truncated_data_is_refused() {
        let error = Gzip.decode(b"Hello", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidCompressedData);
        assert!(error.message.contains("1f 8b"), "{}", error.message);
        assert_eq!(
            Zlib.decode(b"Hello", &Options::new()).unwrap_err().code,
            ErrorCode::InvalidCompressedData
        );

        for codec in all() {
            let compressed = codec.encode(HELLO, &Options::new()).unwrap();
            let truncated = &compressed[..compressed.len() / 2];
            assert_eq!(
                codec.decode(truncated, &Options::new()).unwrap_err().code,
                ErrorCode::InvalidCompressedData,
                "{}",
                codec.meta().id
            );
        }
    }

    #[test]
    fn decompression_stops_at_the_limit() {
        let zeros = vec![0u8; 10_000];
        let bomb = Gzip
            .encode(&zeros, &Options::new().with("level", 9))
            .unwrap();
        assert!(bomb.len() < 100, "{} octets", bomb.len());

        let error = inflate(MultiGzDecoder::new(bomb.as_slice()), "gzip", 1_000).unwrap_err();
        assert_eq!(error.code, ErrorCode::OutputTooLarge);
        assert_eq!(
            inflate(MultiGzDecoder::new(bomb.as_slice()), "gzip", 10_000).unwrap(),
            zeros
        );
    }
}
