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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_exitcodes() {
//         assert_eq!(i32::from(ExitCode::Success), 0);
//         assert_eq!(i32::from(ExitCode::HasResults(true)), 0);
//         assert_eq!(i32::from(ExitCode::HasResults(false)), 1);
//         assert_eq!(i32::from(ExitCode::GeneralError), 1);
//         assert_eq!(i32::from(ExitCode::KilledBySigint), 130);
//     }

//     #[test]
//     fn test_merge_exitcodes() {
//         assert_eq!(
//             merge_exitcodes(vec![ExitCode::Success, ExitCode::Success]),
//             ExitCode::Success
//         );
//         assert_eq!(
//             merge_exitcodes(vec![ExitCode::Success, ExitCode::GeneralError]),
//             ExitCode::GeneralError
//         );
//         assert_eq!(
//             merge_exitcodes(vec![ExitCode::GeneralError, ExitCode::GeneralError]),
//             ExitCode::GeneralError
//         );
//     }
// }
