use super::ffi::*;
use super::Alert;
use std;

#[derive(Clone, Copy, Debug)]
pub enum Error {
  BadParam,
  BadState,
  UnsupportedVersion,
  BadVersion,
  BadLength,
  TooLarge,
  BadMac,
  NoRandom,
  UnknownType,
  Unexpected,
  BadCcs,
  BadAlert,
  BadHandshake,
  OversizedId,
  BadCipherSuite,
  BadCompression,
  BadFraglen,
  BadSecreneg,
  ExtraExtension,
  BadSni,
  BadHelloDone,
  LimitExceeded,
  BadFinished,
  ResumeMismatch,
  InvalidAlgorithm,
  BadSignature,
  WrongKeyUsage,
  NoClientAuth,
  Io,
  RecvFatalAlert(Alert),
  SendFatalAlert(Alert),
  Other(u16)
}

impl Error {
  pub fn as_io_error(self, msg: &'static str) -> std::io::Error {
    println!("as_io_error from {:?}", self);
    std::io::Error::new(std::io::ErrorKind::Other, msg)
  }

  pub fn from_primitive(err: u32) -> Error {
    match err {
      BR_ERR_BAD_PARAM => Error::BadParam,
      BR_ERR_BAD_STATE => Error::BadState,
      BR_ERR_UNSUPPORTED_VERSION => Error::UnsupportedVersion,
      BR_ERR_BAD_VERSION => Error::BadVersion,
      BR_ERR_BAD_LENGTH => Error::BadLength,
      BR_ERR_TOO_LARGE => Error::TooLarge,
      BR_ERR_BAD_MAC => Error::BadMac,
      BR_ERR_NO_RANDOM => Error::NoRandom,
      BR_ERR_UNKNOWN_TYPE => Error::UnknownType,
      BR_ERR_UNEXPECTED => Error::Unexpected,
      BR_ERR_BAD_CCS => Error::BadCcs,
      BR_ERR_BAD_ALERT => Error::BadAlert,
      BR_ERR_BAD_HANDSHAKE => Error::BadHandshake,
      BR_ERR_OVERSIZED_ID => Error::OversizedId,
      BR_ERR_BAD_CIPHER_SUITE => Error::BadCipherSuite,
      BR_ERR_BAD_COMPRESSION => Error::BadCompression,
      BR_ERR_BAD_FRAGLEN => Error::BadFraglen,
      BR_ERR_BAD_SECRENEG => Error::BadSecreneg,
      BR_ERR_EXTRA_EXTENSION => Error::ExtraExtension,
      BR_ERR_BAD_SNI => Error::BadSni,
      BR_ERR_BAD_HELLO_DONE => Error::BadHelloDone,
      BR_ERR_LIMIT_EXCEEDED => Error::LimitExceeded,
      BR_ERR_BAD_FINISHED => Error::BadFinished,
      BR_ERR_RESUME_MISMATCH => Error::ResumeMismatch,
      BR_ERR_INVALID_ALGORITHM => Error::InvalidAlgorithm,
      BR_ERR_BAD_SIGNATURE => Error::BadSignature,
      BR_ERR_WRONG_KEY_USAGE => Error::WrongKeyUsage,
      BR_ERR_NO_CLIENT_AUTH => Error::NoClientAuth,
      BR_ERR_IO => Error::Io,
      other_err => {
        if (err & BR_ERR_RECV_FATAL_ALERT) != 0 {
          Error::RecvFatalAlert(Alert::from_primitive(other_err ^ BR_ERR_RECV_FATAL_ALERT))
        }
        else if (err & BR_ERR_SEND_FATAL_ALERT) != 0 {
          Error::RecvFatalAlert(Alert::from_primitive(other_err ^ BR_ERR_SEND_FATAL_ALERT))
        }
        else {
          Error::Other(other_err as u16)
        }
      }
    }
  }
}
