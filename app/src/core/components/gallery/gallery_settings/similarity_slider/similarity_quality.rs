#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityQuality {
    ALL,
    LessMatched,
    Matched,
    BetterMatched,
    BestMatched,
}

impl SimilarityQuality {
    const _ALL: [SimilarityQuality; 5] = [
        SimilarityQuality::ALL,
        SimilarityQuality::LessMatched,
        SimilarityQuality::Matched,
        SimilarityQuality::BetterMatched,
        SimilarityQuality::BestMatched,
    ];

    pub fn index(&self) -> u8 {
        match self {
            SimilarityQuality::ALL => 0,
            SimilarityQuality::LessMatched => 1,
            SimilarityQuality::Matched => 2,
            SimilarityQuality::BetterMatched => 3,
            SimilarityQuality::BestMatched => 4,
        }
    }

    pub fn from_index(i: u8) -> Self {
        Self::_ALL[i as usize]
    }

    pub fn len() -> u8 {
        Self::_ALL.len() as u8
    }

    pub fn value(&self) -> f32 {
        match self {
            SimilarityQuality::ALL => 0.0,
            SimilarityQuality::LessMatched => 0.65,
            SimilarityQuality::Matched => 0.73,
            SimilarityQuality::BetterMatched => 0.81,
            SimilarityQuality::BestMatched => 0.89,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SimilarityQuality::ALL => "ALL",
            SimilarityQuality::LessMatched => "Less Matched",
            SimilarityQuality::Matched => "Matched",
            SimilarityQuality::BetterMatched => "Better Matched",
            SimilarityQuality::BestMatched => "Best Matched",
        }
    }
}

impl Default for SimilarityQuality {
    fn default() -> Self {
        SimilarityQuality::Matched
    }
}
