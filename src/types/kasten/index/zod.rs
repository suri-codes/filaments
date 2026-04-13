#[derive(Debug, Clone)]
pub struct ZettelOnDisk {
    pub fm: FrontMatter,
    pub body: Body,
    pub path: PathBuf,
}
