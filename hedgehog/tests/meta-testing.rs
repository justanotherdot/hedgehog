//! Meta testing - using Hedgehog to test Hedgehog itself
//!
//! This module contains properties that validate the correctness of Hedgehog's
//! generators, shrinking, and property testing infrastructure.

use hedgehog::*;

#[path = "meta-testing/generator-invariants.rs"]
mod generator_invariants;

#[path = "meta-testing/shrinking-properties.rs"]
mod shrinking_properties;

#[path = "meta-testing/distribution-validation.rs"]
mod distribution_validation;

#[path = "meta-testing/performance-properties.rs"]
mod performance_properties;

#[path = "meta-testing/combinator-properties.rs"]
mod combinator_properties;

#[path = "meta-testing/edge-case-properties.rs"]
mod edge_case_properties;

#[path = "meta-testing/integration-properties.rs"]
mod integration_properties;

#[path = "meta-testing/targeted-properties.rs"]
mod targeted_properties;

#[path = "meta-testing/parallel-properties.rs"]
mod parallel_properties;

#[path = "meta-testing/state-machine-properties.rs"]
mod state_machine_properties;

#[path = "meta-testing/string-properties.rs"]
mod string_properties;

#[path = "meta-testing/result-option-properties.rs"]
mod result_option_properties;

#[path = "meta-testing/composition-properties.rs"]
mod composition_properties;

#[path = "meta-testing/corpus-properties.rs"]
mod corpus_properties;

/// Helper to generate sizes for meta testing  
fn arbitrary_size() -> Gen<Size> {
    Gen::<usize>::from_range(Range::new(0, 20)).map(Size::new)
}

/// Helper to generate seeds for meta testing
fn arbitrary_seed() -> Gen<Seed> {
    Gen::<u64>::from_range(Range::new(0, 10000)).map(Seed::from_u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_test_generator_size_bounds() {
        generator_invariants::test_generator_size_bounds();
    }

    #[test]
    fn meta_test_shrinking_convergence() {
        shrinking_properties::test_shrinking_convergence();
    }

    #[test]
    fn meta_test_distribution_accuracy() {
        distribution_validation::test_frequency_weights();
    }

    #[test]
    fn meta_test_performance_bounds() {
        performance_properties::test_generation_time_bounds();
    }

    #[test]
    fn meta_test_combinator_laws() {
        combinator_properties::test_map_composition();
    }

    #[test]
    fn meta_test_edge_cases() {
        edge_case_properties::test_single_element_ranges();
    }

    #[test]
    fn meta_test_integration_workflows() {
        integration_properties::test_simple_failing_property_workflow();
    }

    #[test]
    fn meta_test_targeted_testing() {
        targeted_properties::test_targeted_search_convergence();
    }

    #[test]
    fn meta_test_parallel_testing() {
        parallel_properties::test_parallel_work_distribution();
    }

    #[test]
    fn meta_test_state_machine_testing() {
        state_machine_properties::test_simple_state_machine_execution();
    }

    #[test]
    fn meta_test_string_generators() {
        string_properties::test_character_generator_ranges();
    }

    #[test]
    fn meta_test_result_option_generators() {
        result_option_properties::test_option_generation_distribution();
    }

    #[test]
    fn meta_test_composition_patterns() {
        composition_properties::test_recursive_composition();
    }

    #[test]
    fn meta_test_corpus_examples() {
        corpus_properties::test_web_input_validation_with_corpus();
        corpus_properties::test_i18n_text_processing_with_glass();
    }
}
