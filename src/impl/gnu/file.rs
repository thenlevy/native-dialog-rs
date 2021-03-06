use super::{should_use, Error, UseCommand};
use crate::{
    r#impl::OpenDialogTarget, Dialog, OpenMultipleFile, OpenSingleDir, OpenSingleFile, Result,
};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;

impl Dialog for OpenSingleFile<'_> {
    type Output = Option<PathBuf>;

    fn show(self) -> Result<Self::Output> {
        match should_use() {
            Some(UseCommand::KDialog(command)) => {
                dialog_implementation_kdialog(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: self.filter,
                    multiple: false,
                    target: OpenDialogTarget::File,
                })
            }
            Some(UseCommand::Zenity(command)) => {
                dialog_implementation_zenity(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: self.filter,
                    multiple: false,
                    target: OpenDialogTarget::File,
                })
            }
            None => Err(Error::NoImplementation),
        }
        .map(|ok| ok.as_deref().map(trim_newlines).map(to_path_buf))
    }
}

impl Dialog for OpenMultipleFile<'_> {
    type Output = Vec<PathBuf>;

    fn show(self) -> Result<Self::Output> {
        let lf_separated = match should_use() {
            Some(UseCommand::KDialog(command)) => {
                dialog_implementation_kdialog(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: self.filter,
                    multiple: true,
                    target: OpenDialogTarget::File,
                })
            }
            Some(UseCommand::Zenity(command)) => {
                dialog_implementation_zenity(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: self.filter,
                    multiple: true,
                    target: OpenDialogTarget::File,
                })
            }
            None => Err(Error::NoImplementation),
        };

        match lf_separated {
            Ok(Some(output)) => {
                let paths = output
                    .split(|c| *c == b'\n')
                    .filter(|c| !c.is_empty())
                    .map(to_path_buf)
                    .collect();
                Ok(paths)
            }
            Ok(None) => Ok(vec![]),
            Err(e) => Err(e),
        }
    }
}

impl Dialog for OpenSingleDir<'_> {
    type Output = Option<PathBuf>;

    fn show(self) -> Result<Self::Output> {
        match should_use() {
            Some(UseCommand::KDialog(command)) => {
                dialog_implementation_kdialog(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: None,
                    multiple: false,
                    target: OpenDialogTarget::Directory,
                })
            }
            Some(UseCommand::Zenity(command)) => {
                dialog_implementation_zenity(ImplementationParams {
                    command,
                    dir: self.dir,
                    filter: None,
                    multiple: false,
                    target: OpenDialogTarget::Directory,
                })
            }
            None => Err(Error::NoImplementation),
        }
        .map(|ok| ok.as_deref().map(trim_newlines).map(to_path_buf))
    }
}

fn trim_newlines(s: &[u8]) -> &[u8] {
    fn is_not_newline(c: &u8) -> bool {
        *c != b'\n'
    }

    let s = s.as_ref();
    if let Some(first) = s.iter().position(is_not_newline) {
        let last = s.iter().rposition(is_not_newline).unwrap();
        &s[first..last + 1]
    } else {
        &[]
    }
}

fn to_path_buf(buf: impl AsRef<[u8]>) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(buf.as_ref()))
}

struct ImplementationParams<'a> {
    command: Command,
    dir: Option<&'a str>,
    filter: Option<&'a [&'a str]>,
    multiple: bool,
    target: OpenDialogTarget,
}

fn dialog_implementation_kdialog(mut params: ImplementationParams) -> Result<Option<Vec<u8>>> {
    let command = &mut params.command;

    match params.target {
        OpenDialogTarget::File => command.arg("--getopenfilename"),
        OpenDialogTarget::Directory => command.arg("--getexistingdirectory"),
    };

    match params.dir {
        Some(dir) => command.arg(dir),
        None => command.arg(""),
    };

    if params.multiple {
        command.args(&["--multiple", "--separate-output"]);
    }

    if let Some(filter) = params.filter {
        let types: Vec<String> = filter.iter().map(|s| format!("*.{}", s)).collect();
        command.arg(types.join(" "));
    }

    let output = command.output()?;

    match output.status.code() {
        Some(0) => Ok(Some(output.stdout)),
        Some(1) => Ok(None),
        _ => Err(Error::UnexpectedOutput("kdialog")),
    }
}

fn dialog_implementation_zenity(mut params: ImplementationParams) -> Result<Option<Vec<u8>>> {
    let command = &mut params.command;

    command.arg("--file-selection");

    if params.target == OpenDialogTarget::Directory {
        command.arg("--directory");
    }

    if params.multiple {
        command.args(&["--multiple", "--separator", "\n"]);
    }

    command.arg("--filename");

    match params.dir {
        Some(dir) => command.arg(dir),
        None => command.arg(""),
    };

    if let Some(filter) = params.filter {
        let types: Vec<String> = filter.iter().map(|s| format!("*.{}", s)).collect();
        command.arg("--file-filter");
        command.arg(types.join(" "));
    }

    let output = command.output()?;

    match output.status.code() {
        Some(0) => Ok(Some(output.stdout)),
        Some(1) => Ok(None),
        _ => Err(Error::UnexpectedOutput("zenity")),
    }
}
