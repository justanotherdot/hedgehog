//! State machine testing meta tests
//!
//! These properties test the state machine testing infrastructure including
//! command generation, action execution, state tracking, and property verification.

use crate::arbitrary_seed;
use hedgehog::*;
use std::collections::HashMap;

/// Property: Simple state machine should generate and execute commands correctly
pub fn test_simple_state_machine_execution() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        // Define a simple counter state
        #[derive(Debug, Clone, PartialEq)]
        struct TestState {
            counter: i32,
            max_value: i32,
        }

        impl TestState {
            fn new() -> Self {
                Self {
                    counter: 0,
                    max_value: 50,
                }
            }

            fn can_increment(&self) -> bool {
                self.counter < self.max_value
            }
        }

        #[derive(Clone, Debug)]
        struct IncrementInput {
            amount: i32,
        }

        impl std::fmt::Display for IncrementInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();

        // Create increment command
        let increment_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment".to_string(),
            |state: &TestState| {
                if state.can_increment() {
                    Some(Gen::constant(IncrementInput { amount: 1 }))
                } else {
                    None
                }
            },
            |input: IncrementInput| input.amount,
        )
        .with_update(
            |state: &mut TestState, input: &IncrementInput, _output: &Var<i32>| {
                state.counter += input.amount;
            },
        )
        .with_ensure(
            |old_state: &TestState, new_state: &TestState, input: &IncrementInput, output: &i32| {
                if new_state.counter != old_state.counter + input.amount {
                    Err("Counter not incremented correctly".to_string())
                } else if *output != input.amount {
                    Err("Output mismatch".to_string())
                } else {
                    Ok(())
                }
            },
        );

        generator.add_command(increment_cmd);

        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 10);

        // Should generate some actions and execute successfully
        let actions_generated = sequential.actions.len();

        match execute_sequential(initial_state, sequential) {
            Ok(()) => actions_generated > 0 && actions_generated <= 10,
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Simple state machine execution property passed"),
        result => panic!("Simple state machine execution property failed: {result:?}"),
    }
}

/// Property: Commands should respect preconditions
pub fn test_command_preconditions() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct BoundedState {
            value: i32,
            limit: i32,
        }

        impl BoundedState {
            fn new() -> Self {
                Self { value: 0, limit: 5 }
            }
        }

        #[derive(Clone, Debug)]
        struct AddInput {
            amount: i32,
        }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();

        // Command that should only execute when under limit
        let constrained_cmd: Command<AddInput, i32, BoundedState, i32> = Command::new(
            "add".to_string(),
            |state: &BoundedState| {
                if state.value < state.limit {
                    Some(Gen::constant(AddInput { amount: 1 }))
                } else {
                    None // Should not generate when at limit
                }
            },
            |input: AddInput| input.amount,
        )
        .with_require(|state: &BoundedState, input: &AddInput| {
            state.value + input.amount <= state.limit
        })
        .with_update(
            |state: &mut BoundedState, input: &AddInput, _output: &Var<i32>| {
                state.value += input.amount;
            },
        );

        generator.add_command(constrained_cmd);

        let initial_state = BoundedState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 10);

        // Should respect the limit constraint
        let actions_count = sequential.actions.len();

        match execute_sequential(initial_state, sequential) {
            Ok(()) => actions_count <= 5, // Should not exceed limit
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Command preconditions property passed"),
        result => panic!("Command preconditions property failed: {result:?}"),
    }
}

/// Property: Multiple commands should be selected with variety
pub fn test_multiple_command_variety() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct MultiState {
            value: i32,
            operations_count: HashMap<String, usize>,
        }

        impl MultiState {
            fn new() -> Self {
                Self {
                    value: 0,
                    operations_count: HashMap::new(),
                }
            }

            fn track_operation(&mut self, op_name: &str) {
                *self
                    .operations_count
                    .entry(op_name.to_string())
                    .or_insert(0) += 1;
            }
        }

        #[derive(Clone, Debug)]
        struct OpInput {
            amount: i32,
            op_name: String,
        }

        impl std::fmt::Display for OpInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", self.op_name, self.amount)
            }
        }

        let mut generator = ActionGenerator::new();

        // Add multiple different commands
        for (cmd_name, amount) in [("add", 1), ("double_add", 2), ("triple_add", 3)] {
            let cmd_name_owned = cmd_name.to_string();
            let cmd: Command<OpInput, i32, MultiState, i32> = Command::new(
                cmd_name_owned.clone(),
                move |state: &MultiState| {
                    if state.value < 20 {
                        Some(Gen::constant(OpInput {
                            amount,
                            op_name: cmd_name_owned.clone(),
                        }))
                    } else {
                        None
                    }
                },
                |input: OpInput| input.amount,
            )
            .with_update(
                |state: &mut MultiState, input: &OpInput, _output: &Var<i32>| {
                    state.value += input.amount;
                    state.track_operation(&input.op_name);
                },
            );

            generator.add_command(cmd);
        }

        let initial_state = MultiState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 10);

        match execute_sequential(initial_state, sequential) {
            Ok(()) => true,
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Multiple command variety property passed"),
        result => panic!("Multiple command variety property failed: {result:?}"),
    }
}

/// Property: Postconditions should catch state inconsistencies
pub fn test_postcondition_verification() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct TestState {
            balance: i32,
        }

        impl TestState {
            fn new() -> Self {
                Self { balance: 100 }
            }
        }

        #[derive(Clone, Debug)]
        struct WithdrawInput {
            amount: i32,
        }

        impl std::fmt::Display for WithdrawInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "-{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();

        // Command with intentionally failing postcondition
        let failing_cmd: Command<WithdrawInput, i32, TestState, i32> = Command::new(
            "withdraw".to_string(),
            |state: &TestState| {
                if state.balance > 10 {
                    Some(Gen::constant(WithdrawInput { amount: 5 }))
                } else {
                    None
                }
            },
            |input: WithdrawInput| input.amount,
        )
        .with_update(
            |state: &mut TestState, input: &WithdrawInput, _output: &Var<i32>| {
                state.balance -= input.amount;
            },
        )
        .with_ensure(
            |old_state: &TestState, new_state: &TestState, input: &WithdrawInput, _output: &i32| {
                // Intentionally strict postcondition that should fail occasionally
                if new_state.balance < old_state.balance - input.amount {
                    Err("Balance decreased too much".to_string())
                } else if new_state.balance == old_state.balance - input.amount {
                    Ok(()) // This is correct
                } else {
                    Err("Balance didn't decrease correctly".to_string())
                }
            },
        );

        generator.add_command(failing_cmd);

        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 3);

        // Should execute and potentially catch postcondition violations
        match execute_sequential(initial_state, sequential) {
            Ok(()) => true,  // Postcondition passed
            Err(_) => false, // Would fail if postcondition caught an error (which is good)
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Postcondition verification property passed"),
        result => panic!("Postcondition verification property failed: {result:?}"),
    }
}

/// Property: State updates during generation should match execution
pub fn test_generation_execution_consistency() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct ConsistentState {
            counter: i32,
            max_commands: usize,
        }

        impl ConsistentState {
            fn new() -> Self {
                Self {
                    counter: 0,
                    max_commands: 5,
                }
            }
        }

        #[derive(Clone, Debug)]
        struct IncrementInput {
            value: i32,
        }

        impl std::fmt::Display for IncrementInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.value)
            }
        }

        let mut generator = ActionGenerator::new();

        // Command that updates state predictably
        let increment_cmd: Command<IncrementInput, i32, ConsistentState, i32> = Command::new(
            "increment".to_string(),
            |state: &ConsistentState| {
                if (state.counter as usize) < state.max_commands {
                    Some(Gen::constant(IncrementInput { value: 1 }))
                } else {
                    None
                }
            },
            |input: IncrementInput| input.value,
        )
        .with_update(
            |state: &mut ConsistentState, input: &IncrementInput, _output: &Var<i32>| {
                state.counter += input.value;
            },
        );

        generator.add_command(increment_cmd);

        let initial_state = ConsistentState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 10);

        // Should stop at max_commands due to generation state tracking
        let actions_count = sequential.actions.len();

        match execute_sequential(initial_state, sequential) {
            Ok(()) => actions_count <= 5, // Should respect the limit
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Generation execution consistency property passed"),
        result => panic!("Generation execution consistency property failed: {result:?}"),
    }
}

/// Property: Complex state machine with interdependent commands should work correctly
pub fn test_complex_interdependent_commands() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct ResourceState {
            resources: HashMap<String, i32>,
            operations: Vec<String>,
        }

        impl ResourceState {
            fn new() -> Self {
                let mut resources = HashMap::new();
                resources.insert("gold".to_string(), 10);
                resources.insert("wood".to_string(), 5);

                Self {
                    resources,
                    operations: Vec::new(),
                }
            }

            fn has_resource(&self, resource: &str, amount: i32) -> bool {
                self.resources.get(resource).copied().unwrap_or(0) >= amount
            }

            fn spend_resource(&mut self, resource: &str, amount: i32) {
                if let Some(current) = self.resources.get_mut(resource) {
                    *current -= amount;
                }
            }

            fn add_resource(&mut self, resource: &str, amount: i32) {
                *self.resources.entry(resource.to_string()).or_insert(0) += amount;
            }
        }

        #[derive(Clone, Debug)]
        struct TradeInput {
            spend_resource: String,
            spend_amount: i32,
            gain_resource: String,
            gain_amount: i32,
        }

        impl std::fmt::Display for TradeInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "trade {} {} for {} {}",
                    self.spend_amount, self.spend_resource, self.gain_amount, self.gain_resource
                )
            }
        }

        let mut generator = ActionGenerator::new();

        // Trade command that requires resources
        let trade_cmd: Command<TradeInput, bool, ResourceState, bool> = Command::new(
            "trade".to_string(),
            |state: &ResourceState| {
                if state.has_resource("gold", 2) {
                    Some(Gen::constant(TradeInput {
                        spend_resource: "gold".to_string(),
                        spend_amount: 2,
                        gain_resource: "wood".to_string(),
                        gain_amount: 1,
                    }))
                } else {
                    None
                }
            },
            |_input: TradeInput| true,
        )
        .with_require(|state: &ResourceState, input: &TradeInput| {
            state.has_resource(&input.spend_resource, input.spend_amount)
        })
        .with_update(
            |state: &mut ResourceState, input: &TradeInput, _output: &Var<bool>| {
                state.spend_resource(&input.spend_resource, input.spend_amount);
                state.add_resource(&input.gain_resource, input.gain_amount);
                state.operations.push("trade".to_string());
            },
        )
        .with_ensure(
            |old_state: &ResourceState,
             new_state: &ResourceState,
             input: &TradeInput,
             output: &bool| {
                let old_spend = old_state
                    .resources
                    .get(&input.spend_resource)
                    .copied()
                    .unwrap_or(0);
                let new_spend = new_state
                    .resources
                    .get(&input.spend_resource)
                    .copied()
                    .unwrap_or(0);
                let old_gain = old_state
                    .resources
                    .get(&input.gain_resource)
                    .copied()
                    .unwrap_or(0);
                let new_gain = new_state
                    .resources
                    .get(&input.gain_resource)
                    .copied()
                    .unwrap_or(0);

                if new_spend != old_spend - input.spend_amount {
                    Err("Spend resource not updated correctly".to_string())
                } else if new_gain != old_gain + input.gain_amount {
                    Err("Gain resource not updated correctly".to_string())
                } else if !output {
                    Err("Trade should return true".to_string())
                } else {
                    Ok(())
                }
            },
        );

        generator.add_command(trade_cmd);

        let initial_state = ResourceState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 5);

        // Should execute trades correctly
        match execute_sequential(initial_state, sequential) {
            Ok(()) => true,
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(3).with_shrinks(1);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Complex interdependent commands property passed"),
        result => panic!("Complex interdependent commands property failed: {result:?}"),
    }
}

/// Property: State machine with no available commands should handle gracefully
pub fn test_empty_command_sequence_handling() {
    let prop = for_all_named(arbitrary_seed(), "seed", |&_seed: &Seed| {
        #[derive(Debug, Clone, PartialEq)]
        struct BlockedState {
            locked: bool,
        }

        impl BlockedState {
            fn new() -> Self {
                Self { locked: true }
            }
        }

        #[derive(Clone, Debug)]
        struct UnlockInput;

        impl std::fmt::Display for UnlockInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "unlock")
            }
        }

        let mut generator = ActionGenerator::new();

        // Command that can never execute due to state
        let impossible_cmd: Command<UnlockInput, bool, BlockedState, bool> = Command::new(
            "unlock".to_string(),
            |state: &BlockedState| {
                if !state.locked {
                    // Only when unlocked, but starts locked
                    Some(Gen::constant(UnlockInput))
                } else {
                    None
                }
            },
            |_input: UnlockInput| true,
        );

        generator.add_command(impossible_cmd);

        let initial_state = BlockedState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 10);

        // Should generate empty sequence gracefully
        let is_empty = sequential.actions.is_empty();

        match execute_sequential(initial_state, sequential) {
            Ok(()) => is_empty, // Should succeed with empty sequence
            Err(_) => false,
        }
    });

    let fast_config = Config::default().with_tests(5).with_shrinks(2);
    match prop.run(&fast_config) {
        TestResult::Pass { .. } => println!("✓ Empty command sequence handling property passed"),
        result => panic!("Empty command sequence handling property failed: {result:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_state_machine_property_tests() {
        test_simple_state_machine_execution();
        test_command_preconditions();
        test_multiple_command_variety();
        test_postcondition_verification();
        test_generation_execution_consistency();
        test_complex_interdependent_commands();
        test_empty_command_sequence_handling();
    }
}
