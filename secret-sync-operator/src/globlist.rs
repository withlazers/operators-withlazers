pub use globset::Error;

pub struct GlobList(Vec<globset::GlobMatcher>);

impl GlobList {
    pub fn new(globs: &str) -> Result<Self, Error> {
        // Ok(GlobList(
        //     globs
        //         .split(' ')
        //         .filter(|s| !s.is_empty())
        //         .map(|s| globset::Glob::new(s).map(|g| g.compile_matcher()))
        //         .try_collect::<Vec<_>>()?,
        // ))
        let globs = globs
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| globset::Glob::new(s).map(|g| g.compile_matcher()))
            .collect::<Vec<_>>();
        let mut result = Vec::with_capacity(globs.len());

        for g in globs {
            result.push(g?);
        }

        Ok(Self(result))
    }

    pub fn is_match(&self, string: &str) -> bool {
        self.0.iter().any(|g| g.is_match(string))
    }
}
