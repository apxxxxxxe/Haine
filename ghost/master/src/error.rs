use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShioriError {
  UndefinedVariable,
  ParseIntError,
  SystemTimeError,
  FieldAccessError,
  ArrayAccessError,
  TranslaterNotReadyError,
  TalkNotFound,
  ParseRequestError,
  NotSetScopeError(String),
  BadRequest,
  FileWriteError,
  InvalidEvent,
}

impl fmt::Display for ShioriError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ShioriError::UndefinedVariable => {
        write!(f, "[UndefinedVariable]未定義の変数にアクセスしました")
      }
      ShioriError::ParseIntError => write!(f, "[ParseIntError]文字列のパースに失敗しました"),
      ShioriError::SystemTimeError => {
        write!(f, "[SystemTimeError]システム時刻の取得に失敗しました")
      }
      ShioriError::FieldAccessError => write!(
        f,
        "[FieldAccessError]変数フィールドへのアクセスに失敗しました"
      ),
      ShioriError::ArrayAccessError => {
        write!(f, "[ArrayAccessError]配列の範囲外アクセスが発生しました")
      }
      ShioriError::TranslaterNotReadyError => write!(
        f,
        "[TranslaterNotReadyError]トランスレータのセットアップが完了していません"
      ),
      ShioriError::TalkNotFound => write!(f, "[TalkNotFound]指定されたトークが見つかりません"),
      ShioriError::ParseRequestError => write!(
        f,
        "[ParseRequestError]SHIORIリクエストのパースに失敗しました"
      ),
      ShioriError::NotSetScopeError(v) => write!(
        f,
        "[NotSetScopeError]次のスクリプトの頭にスコープ指定がありません: {}",
        v
      ),
      ShioriError::BadRequest => write!(f, "[BadRequest]リクエストが不正です"),
      ShioriError::FileWriteError => write!(f, "[FileWriteError]ファイルの書き込みに失敗しました"),
      ShioriError::InvalidEvent => write!(f, "[InvalidEvent]無効なイベントが指定されました"),
    }
  }
}

impl Error for ShioriError {}
