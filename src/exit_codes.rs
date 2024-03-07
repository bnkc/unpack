#![allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success,
    HasResults(bool),
    GeneralError,
    KilledBySigint,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        match code {
            ExitCode::Success => 0,
            ExitCode::HasResults(has_results) => !has_results as i32,
            ExitCode::GeneralError => 1,
            ExitCode::KilledBySigint => 130,
        }
    }
}

impl ExitCode {
    fn is_error(self) -> bool {
        i32::from(self) != 0
    }

    /// Exit the process with the appropriate code.
    pub fn exit(self) -> ! {
        std::process::exit(self.into())
    }
}

pub fn merge_exitcodes(results: impl IntoIterator<Item = ExitCode>) -> ExitCode {
    if results.into_iter().any(ExitCode::is_error) {
        return ExitCode::GeneralError;
    }
    ExitCode::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_when_no_results() {
        assert_eq!(merge_exitcodes([]), ExitCode::Success);
    }

    #[test]
    fn general_error_when_any_error() {
        assert_eq!(
            merge_exitcodes([ExitCode::Success, ExitCode::GeneralError]),
            ExitCode::GeneralError
        );
        assert_eq!(
            merge_exitcodes([ExitCode::GeneralError, ExitCode::Success]),
            ExitCode::GeneralError
        );

        assert_eq!(
            merge_exitcodes([ExitCode::GeneralError, ExitCode::GeneralError]),
            ExitCode::GeneralError
        );
    }
}
