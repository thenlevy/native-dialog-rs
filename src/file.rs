pub struct FileType<'a> {
    pub description: &'a str,
    pub extensions: &'a [&'a str],
}

impl FileType<'_> {
    pub const fn new<'a>(description: &'a str, extensions: &'a [&'a str]) -> FileType<'a> {
        FileType {
            description,
            extensions,
        }
    }
}

pub struct OpenSingleFile<'a> {
    pub dir: Option<&'a str>,
    pub filter: Option<&'a [&'a str]>,
}

pub struct OpenMultipleFile<'a> {
    pub dir: Option<&'a str>,
    pub filter: Option<&'a [&'a str]>,
}

pub struct SaveFile<'a> {
    pub dir: Option<&'a str>,
    pub name: &'a str,
    pub types: &'a [FileType<'a>],
}
