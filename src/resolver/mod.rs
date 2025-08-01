pub mod file_classifier;
pub mod dependency_detector;
pub mod terminal_detector;

pub use file_classifier::{FileClassifier, FileClassification, FileInfo, FileContext, FileUsage};
pub use dependency_detector::{DependencyResolver, Dependency, DependencyType};
pub use terminal_detector::{TerminalDetector, TerminalAnalysis, TerminalRequirement, TerminalFeature};