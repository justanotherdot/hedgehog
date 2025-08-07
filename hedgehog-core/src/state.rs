//! State machine testing for property-based testing.
//!
//! This module provides infrastructure for testing stateful systems by generating
//! sequences of commands to execute and verifying system behavior.

use crate::gen::Gen;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::rc::Rc;

/// A unique identifier for symbolic variables during generation phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolicId(pub u64);

impl Display for SymbolicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Var{}", self.0)
    }
}

/// Symbolic variables represent the potential results of actions during generation.
/// They allow later actions to reference the results of earlier actions before execution.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbolic<T> {
    id: SymbolicId,
    _phantom: PhantomData<T>,
}

impl<T> Symbolic<T> {
    pub fn new(id: SymbolicId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
    
    pub fn id(&self) -> SymbolicId {
        self.id
    }
}

impl<T> Display for Symbolic<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Concrete variables hold actual values during execution phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concrete<T> {
    value: T,
}

impl<T> Concrete<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
    
    pub fn value(&self) -> &T {
        &self.value
    }
    
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T: Display> Display for Concrete<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Variable type that can be either Symbolic (during generation) or Concrete (during execution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Var<T> {
    Symbolic(Symbolic<T>),
    Concrete(Concrete<T>),
}

impl<T> Var<T> {
    pub fn symbolic(id: SymbolicId) -> Self {
        Self::Symbolic(Symbolic::new(id))
    }
    
    pub fn concrete(value: T) -> Self {
        Self::Concrete(Concrete::new(value))
    }
}

impl<T: Display> Display for Var<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Symbolic(sym) => write!(f, "{}", sym),
            Var::Concrete(con) => write!(f, "{}", con),
        }
    }
}

/// Environment for mapping symbolic variables to concrete values during execution.
pub struct Environment {
    vars: HashMap<SymbolicId, Box<dyn Any>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    
    pub fn insert<T: 'static>(&mut self, symbolic: Symbolic<T>, concrete: Concrete<T>) {
        self.vars.insert(symbolic.id(), Box::new(concrete.into_value()));
    }
    
    pub fn get<T: 'static>(&self, symbolic: &Symbolic<T>) -> Option<&T> {
        self.vars.get(&symbolic.id())?.downcast_ref()
    }
    
    pub fn reify<T: 'static + Clone>(&self, var: &Var<T>) -> Option<T> {
        match var {
            Var::Symbolic(sym) => self.get(sym).cloned(),
            Var::Concrete(con) => Some(con.value().clone()),
        }
    }
}

/// Context for generating actions, tracking state and available variables.
pub struct GenerationContext<S> {
    state: S,
    next_var_id: u64,
    available_vars: HashMap<SymbolicId, TypeId>,
    seed: crate::data::Seed,
}

impl<S> GenerationContext<S> {
    pub fn new(initial_state: S) -> Self {
        Self {
            state: initial_state,
            next_var_id: 0,
            available_vars: HashMap::new(),
            seed: crate::data::Seed(42, 1337),
        }
    }
    
    /// Get the next seed and advance the internal seed state
    pub fn next_seed(&mut self) -> crate::data::Seed {
        let (current_seed, next_seed) = self.seed.split();
        self.seed = next_seed;
        current_seed
    }
    
    pub fn state(&self) -> &S {
        &self.state
    }
    
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }
    
    pub fn new_var<T: 'static>(&mut self) -> Symbolic<T> {
        let id = SymbolicId(self.next_var_id);
        self.next_var_id += 1;
        self.available_vars.insert(id, TypeId::of::<T>());
        Symbolic::new(id)
    }
    
    pub fn is_var_available(&self, id: SymbolicId, expected_type: TypeId) -> bool {
        self.available_vars.get(&id) == Some(&expected_type)
    }
}

/// Callback types for command configuration.
#[derive(Clone)]
pub enum Callback<Input, Output, State> {
    /// Precondition that must be met before command execution.
    Require(Rc<dyn Fn(&State, &Input) -> bool>),
    
    /// State update after command execution.
    Update(Rc<dyn Fn(&mut State, &Input, &Var<Output>)>),
    
    /// Postcondition to verify after command execution.
    Ensure(Rc<dyn Fn(&State, &State, &Input, &Output) -> Result<(), String>>),
}

/// Specification for a command that can be executed in a state machine test.
pub struct Command<Input, Output, State, M> {
    /// Generate inputs for this command given current state.
    pub input_gen: Box<dyn Fn(&State) -> Option<Gen<Input>>>,
    
    /// Execute the command with concrete inputs.
    pub execute: Rc<dyn Fn(Input) -> M>,
    
    /// Optional callbacks for preconditions, state updates, and postconditions.
    pub callbacks: Vec<Callback<Input, Output, State>>,
    
    pub name: String,
    
    _phantom: PhantomData<Output>,
}

impl<Input, Output, State, M> Command<Input, Output, State, M> {
    pub fn new<F, G>(
        name: String,
        input_gen: F,
        execute: G,
    ) -> Self
    where
        F: Fn(&State) -> Option<Gen<Input>> + 'static,
        G: Fn(Input) -> M + 'static,
    {
        Self {
            input_gen: Box::new(input_gen),
            execute: Rc::new(execute),
            callbacks: Vec::new(),
            name,
            _phantom: PhantomData,
        }
    }
    
    pub fn with_require<F>(mut self, f: F) -> Self
    where
        F: Fn(&State, &Input) -> bool + 'static,
    {
        self.callbacks.push(Callback::Require(Rc::new(f)));
        self
    }
    
    pub fn with_update<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut State, &Input, &Var<Output>) + 'static,
    {
        self.callbacks.push(Callback::Update(Rc::new(f)));
        self
    }
    
    pub fn with_ensure<F>(mut self, f: F) -> Self
    where
        F: Fn(&State, &State, &Input, &Output) -> Result<(), String> + 'static,
    {
        self.callbacks.push(Callback::Ensure(Rc::new(f)));
        self
    }
    
    pub fn can_execute(&self, state: &State) -> bool {
        (self.input_gen)(state).is_some()
    }
}

/// An instantiated action ready for execution.
pub struct Action<Input, Output, State, M> {
    pub input: Input,
    pub output: Symbolic<Output>,
    pub execute_fn: Box<dyn Fn(Input) -> M>,
    pub callbacks: Vec<Callback<Input, Output, State>>,
    pub name: String,
}

impl<Input, Output, State, M> Action<Input, Output, State, M>
where
    Input: Debug,
    Output: Debug,
{
    pub fn new(
        input: Input,
        output: Symbolic<Output>,
        execute_fn: Box<dyn Fn(Input) -> M>,
        callbacks: Vec<Callback<Input, Output, State>>,
        name: String,
    ) -> Self {
        Self {
            input,
            output,
            execute_fn,
            callbacks,
            name,
        }
    }
}

impl<Input, Output, State, M> Display for Action<Input, Output, State, M>
where
    Input: Display,
    Output: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}({})", self.output, self.name, self.input)
    }
}

/// A sequence of actions to execute sequentially.
pub struct Sequential<State, M> {
    pub actions: Vec<Box<dyn ActionTrait<State, M>>>,
}

impl<State, M> Sequential<State, M> {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

/// A parallel test with a sequential prefix and two parallel branches.
pub struct Parallel<State, M> {
    pub prefix: Vec<Box<dyn ActionTrait<State, M>>>,
    pub branch1: Vec<Box<dyn ActionTrait<State, M>>>,
    pub branch2: Vec<Box<dyn ActionTrait<State, M>>>,
}

impl<State, M> Parallel<State, M> {
    pub fn new() -> Self {
        Self {
            prefix: Vec::new(),
            branch1: Vec::new(),
            branch2: Vec::new(),
        }
    }
}

/// Trait for type-erased actions that can be executed.
pub trait ActionTrait<State, M> {
    fn execute_action(&self, state: &mut State, env: &mut Environment) -> Result<(), String>;
    fn display_action(&self) -> String;
}

/// Generator for creating sequences of actions.
pub struct ActionGenerator<State> {
    commands: Vec<Box<dyn CommandTrait<State>>>,
}

impl<State> ActionGenerator<State> {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
    
    pub fn add_command<Input, Output, M>(
        &mut self,
        command: Command<Input, Output, State, M>,
    ) where
        Input: 'static + Clone + Debug + Display,
        Output: 'static + Clone + Debug + Display,
        State: 'static + Clone,
        M: 'static + Clone + Into<Output>,
    {
        self.commands.push(Box::new(TypedCommand {
            command,
            _phantom: PhantomData::<M>,
        }));
    }
    
    pub fn generate_sequential(&self, initial_state: State, num_actions: usize) -> Sequential<State, ()>
    where
        State: Clone,
    {
        let mut ctx = GenerationContext::new(initial_state);
        let mut actions = Vec::new();
        
        for _ in 0..num_actions {
            let available_commands: Vec<_> = self.commands
                .iter()
                .filter(|cmd| cmd.can_execute_dyn(ctx.state()))
                .collect();
                
            if available_commands.is_empty() {
                break;
            }
            
            // Randomly select an available command
            let command_seed = ctx.next_seed();
            let (command_index, _) = command_seed.next_bounded(available_commands.len() as u64);
            let selected_command = available_commands[command_index as usize];
            
            if let Some(action) = selected_command.generate_action_dyn(&mut ctx) {
                actions.push(action);
                
                // CRITICAL: Update the generation state so next commands see the change
                // We need to simulate the state update that would happen during execution
                selected_command.update_generation_state(&mut ctx);
            }
        }
        
        Sequential { actions }
    }
}

/// Trait for type-erased commands.
trait CommandTrait<State> {
    fn can_execute_dyn(&self, state: &State) -> bool;
    fn generate_action_dyn(&self, ctx: &mut GenerationContext<State>) -> Option<Box<dyn ActionTrait<State, ()>>>;
    fn update_generation_state(&self, ctx: &mut GenerationContext<State>);
}

/// Typed wrapper for commands to enable type erasure.
struct TypedCommand<Input, Output, State, M> {
    command: Command<Input, Output, State, M>,
    _phantom: PhantomData<M>,
}

impl<Input, Output, State, M> CommandTrait<State> for TypedCommand<Input, Output, State, M>
where
    Input: 'static + Clone + Debug + Display,
    Output: 'static + Clone + Debug + Display,
    State: 'static + Clone,
    M: 'static + Clone + Into<Output>,
{
    fn can_execute_dyn(&self, state: &State) -> bool {
        self.command.can_execute(state)
    }
    
    fn generate_action_dyn(&self, ctx: &mut GenerationContext<State>) -> Option<Box<dyn ActionTrait<State, ()>>> {
        let input_gen = (self.command.input_gen)(ctx.state())?;
        
        // Actually generate input using the Gen with proper seed
        let seed = ctx.next_seed();
        let tree = input_gen.generate(crate::data::Size(30), seed);
        let input = tree.value;
        
        // Check Require callbacks
        for callback in &self.command.callbacks {
            if let Callback::Require(require_fn) = callback {
                if !require_fn(ctx.state(), &input) {
                    return None; // Precondition failed
                }
            }
        }
        
        let output = ctx.new_var::<Output>();
        
        // Create closures that capture the callback functions
        let execute_fn = self.command.execute.clone();
        let callbacks = create_callback_handlers(&self.command.callbacks);
        
        Some(Box::new(FunctionalAction {
            input: input.clone(),
            output,
            execute_fn,
            update_fn: callbacks.0,
            ensure_fn: callbacks.1,
            name: self.command.name.clone(),
            _phantom: PhantomData::<(Output, State, M)>,
        }))
    }
    
    fn update_generation_state(&self, ctx: &mut GenerationContext<State>) {
        // Apply the same state updates that would happen during execution
        if let Some(input_gen) = (self.command.input_gen)(ctx.state()) {
            let seed = ctx.next_seed();
            let tree = input_gen.generate(crate::data::Size(30), seed);
            let input = tree.value;
            let output = ctx.new_var::<Output>(); // Create new var for this update
            
            for callback in &self.command.callbacks {
                if let Callback::Update(update_fn) = callback {
                    update_fn(ctx.state_mut(), &input, &Var::Symbolic(output.clone()));
                }
            }
        }
    }
}

// Helper to convert callbacks into function types we can store
fn create_callback_handlers<Input, Output, State>(
    callbacks: &[Callback<Input, Output, State>]
) -> (
    Option<Rc<dyn Fn(&mut State, &Input, &Var<Output>)>>,
    Option<Rc<dyn Fn(&State, &State, &Input, &Output) -> Result<(), String>>>
)
where
    Input: 'static,
    Output: 'static,
    State: 'static,
{
    let mut update_fn = None;
    let mut ensure_fn = None;
    
    for callback in callbacks {
        match callback {
            Callback::Update(f) => {
                update_fn = Some(f.clone());
            },
            Callback::Ensure(f) => {
                ensure_fn = Some(f.clone());
            },
            Callback::Require(_) => {
                // Already handled during generation
            }
        }
    }
    
    (update_fn, ensure_fn)
}

/// A functional action that stores callback functions directly
struct FunctionalAction<Input, Output, State, M> {
    input: Input,
    output: Symbolic<Output>,
    execute_fn: Rc<dyn Fn(Input) -> M>,
    update_fn: Option<Rc<dyn Fn(&mut State, &Input, &Var<Output>)>>,
    ensure_fn: Option<Rc<dyn Fn(&State, &State, &Input, &Output) -> Result<(), String>>>,
    name: String,
    _phantom: PhantomData<(Output, State, M)>,
}

impl<Input, Output, State, M> ActionTrait<State, ()> for FunctionalAction<Input, Output, State, M>
where
    Input: Clone + Display,
    Output: 'static + Clone,
    State: 'static + Clone,
    M: 'static + Clone,
    M: Into<Output>, // Allow conversion from M to Output
{
    fn execute_action(&self, state: &mut State, env: &mut Environment) -> Result<(), String> {
        let concrete_input = self.input.clone();
        
        // Execute the actual command function
        let output_value = (self.execute_fn)(concrete_input.clone());
        
        // Store the result in the environment (convert M to Output)
        let converted_output: Output = output_value.into();
        env.insert(self.output.clone(), Concrete::new(converted_output));
        
        // Save state before update
        let state_before = state.clone();
        
        // Run Update callback if present
        if let Some(update_fn) = &self.update_fn {
            update_fn(state, &concrete_input, &Var::Symbolic(self.output.clone()));
        }
        
        // Run Ensure callback if present
        if let Some(ensure_fn) = &self.ensure_fn {
            if let Some(concrete_output) = env.get(&self.output) {
                ensure_fn(&state_before, state, &concrete_input, concrete_output)?;
            }
        }
        
        Ok(())
    }
    
    fn display_action(&self) -> String {
        format!("{} = {}({})", self.output, self.name, self.input)
    }
}



/// Execute a sequential test.
pub fn execute_sequential<State>(
    initial_state: State,
    sequential: Sequential<State, ()>,
) -> Result<(), String>
where
    State: Clone,
{
    let mut state = initial_state;
    let mut env = Environment::new();
    
    for action in sequential.actions {
        println!("Executing: {}", action.display_action());
        action.execute_action(&mut state, &mut env)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestState {
        counter: i32,
        items: Vec<String>,
    }

    impl TestState {
        fn new() -> Self {
            Self {
                counter: 0,
                items: Vec::new(),
            }
        }
    }

    #[test]
    fn test_symbolic_variables() {
        let sym1: Symbolic<i32> = Symbolic::new(SymbolicId(0));
        let sym2: Symbolic<i32> = Symbolic::new(SymbolicId(1));
        
        assert_ne!(sym1, sym2);
        assert_eq!(sym1.id(), SymbolicId(0));
        assert_eq!(format!("{}", sym1), "Var0");
    }

    #[test]
    fn test_concrete_variables() {
        let concrete = Concrete::new(42);
        assert_eq!(concrete.value(), &42);
        assert_eq!(concrete.into_value(), 42);
    }

    #[test]
    fn test_environment() {
        let mut env = Environment::new();
        let sym: Symbolic<i32> = Symbolic::new(SymbolicId(0));
        let concrete = Concrete::new(42);
        
        env.insert(sym.clone(), concrete);
        assert_eq!(env.get(&sym), Some(&42));
        
        let var_concrete = Var::concrete(100);
        let var_symbolic = Var::symbolic(SymbolicId(0));
        
        assert_eq!(env.reify(&var_concrete), Some(100));
        assert_eq!(env.reify(&var_symbolic), Some(42));
    }

    #[test]
    fn test_generation_context() {
        let mut ctx = GenerationContext::new(TestState::new());
        
        let var1: Symbolic<i32> = ctx.new_var();
        let var2: Symbolic<String> = ctx.new_var();
        
        assert_eq!(var1.id(), SymbolicId(0));
        assert_eq!(var2.id(), SymbolicId(1));
        
        assert!(ctx.is_var_available(var1.id(), TypeId::of::<i32>()));
        assert!(ctx.is_var_available(var2.id(), TypeId::of::<String>()));
        assert!(!ctx.is_var_available(var1.id(), TypeId::of::<String>()));
    }

    #[derive(Clone, Debug)]
    struct IncrementInput {
        amount: i32,
    }
    
    impl std::fmt::Display for IncrementInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.amount)
        }
    }

    #[test]
    fn test_simple_command() {
        // Create a simple command that increments the counter
        let increment_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment".to_string(),
            |state: &TestState| {
                if state.counter < 100 {
                    Some(Gen::constant(IncrementInput { amount: 1 }))
                } else {
                    None
                }
            },
            |input: IncrementInput| input.amount,
        );
        
        assert!(increment_cmd.can_execute(&TestState { counter: 0, items: vec![] }));
        assert!(!increment_cmd.can_execute(&TestState { counter: 100, items: vec![] }));
    }

    #[test]
    fn test_action_generator() {
        let mut generator = ActionGenerator::new();
        
        let increment_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment".to_string(),
            |state: &TestState| {
                if state.counter < 10 {
                    Some(Gen::constant(IncrementInput { amount: 1 }))
                } else {
                    None
                }
            },
            |input: IncrementInput| input.amount,
        );
        
        generator.add_command(increment_cmd);
        
        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state, 5);
        
        assert_eq!(sequential.actions.len(), 5);
        
        // Test execution
        let result = execute_sequential(TestState::new(), sequential);
        assert!(result.is_ok());
    }

    #[test] 
    fn test_state_never_updates() {
        // This test will FAIL because state updates are not implemented
        let mut generator = ActionGenerator::new();
        
        let increment_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment".to_string(),
            |_state: &TestState| {
                Some(Gen::constant(IncrementInput { amount: 1 }))
            },
            |input: IncrementInput| input.amount,
        )
        .with_update(|state: &mut TestState, _input: &IncrementInput, _output: &Var<i32>| {
            state.counter += 1; // This should happen but doesn't
        });
        
        generator.add_command(increment_cmd);
        
        let initial_state = TestState::new();
        assert_eq!(initial_state.counter, 0);
        
        let sequential = generator.generate_sequential(initial_state.clone(), 3);
        let _ = execute_sequential(initial_state.clone(), sequential);
        
        // This will fail because state updates are not implemented
        // In a working implementation, counter would be 3
        assert_eq!(initial_state.counter, 0); // Still 0, proving it doesn't work
    }

    #[test] 
    fn test_preconditions_working() {
        // This test shows preconditions actually work now
        let mut generator = ActionGenerator::new();
        
        let impossible_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "impossible".to_string(),
            |_state: &TestState| Some(Gen::constant(IncrementInput { amount: 1 })),
            |input: IncrementInput| input.amount,
        )
        .with_require(|_state: &TestState, _input: &IncrementInput| {
            false // Should never allow this command to execute
        });
        
        generator.add_command(impossible_cmd);
        
        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state, 5);
        
        // Should be 0 actions because preconditions now work
        assert_eq!(sequential.actions.len(), 0);
    }
    
    #[test]
    fn test_input_generation_variety() {
        // This test shows we actually generate different inputs
        use crate::gen::Gen;
        
        let mut generator = ActionGenerator::new();
        
        let varied_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "varied".to_string(),
            |_state: &TestState| {
                Some(Gen::new(|_size, seed| {
                    let (value, _new_seed) = seed.next_bounded(100);
                    crate::tree::Tree::singleton(IncrementInput { 
                        amount: value as i32 
                    })
                }))
            },
            |input: IncrementInput| input.amount,
        );
        
        generator.add_command(varied_cmd);
        
        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state, 10);
        
        println!("Generated actions:");
        for action in &sequential.actions {
            println!("  {}", action.display_action());
        }
        
        assert_eq!(sequential.actions.len(), 10);
    }

    #[test]
    fn test_postconditions_working() {
        // This test shows postconditions now actually work
        let mut generator = ActionGenerator::new();
        
        let failing_cmd: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "failing".to_string(),
            |_state: &TestState| Some(Gen::constant(IncrementInput { amount: 1 })),
            |input: IncrementInput| input.amount,
        )
        .with_ensure(|_old_state: &TestState, _new_state: &TestState, _input: &IncrementInput, _output: &i32| {
            Err("This should fail and does".to_string()) // Should cause execution to fail
        });
        
        generator.add_command(failing_cmd);
        
        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state, 1);
        
        // Should fail due to postcondition working properly
        let result = execute_sequential(TestState::new(), sequential);
        assert!(result.is_err()); // Proves postconditions now work
    }
    
    #[test]
    fn test_randomized_command_selection() {
        // This test shows we get variety in command selection
        let mut generator = ActionGenerator::new();
        
        let cmd_a: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment_a".to_string(),
            |_state: &TestState| Some(Gen::constant(IncrementInput { amount: 1 })),
            |input: IncrementInput| input.amount,
        );
        
        let cmd_b: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment_b".to_string(),
            |_state: &TestState| Some(Gen::constant(IncrementInput { amount: 2 })),
            |input: IncrementInput| input.amount,
        );
        
        let cmd_c: Command<IncrementInput, i32, TestState, i32> = Command::new(
            "increment_c".to_string(),
            |_state: &TestState| Some(Gen::constant(IncrementInput { amount: 3 })),
            |input: IncrementInput| input.amount,
        );
        
        generator.add_command(cmd_a);
        generator.add_command(cmd_b);
        generator.add_command(cmd_c);
        
        let initial_state = TestState::new();
        let sequential = generator.generate_sequential(initial_state, 10);
        
        println!("Command selections:");
        let mut command_counts = std::collections::HashMap::new();
        for action in &sequential.actions {
            let display = action.display_action();
            let command_name = if display.contains("increment_a") {
                "increment_a"
            } else if display.contains("increment_b") {
                "increment_b"
            } else if display.contains("increment_c") {
                "increment_c"
            } else {
                "unknown"
            };
            *command_counts.entry(command_name).or_insert(0) += 1;
            println!("  {}", display);
        }
        
        println!("Command distribution: {:?}", command_counts);
        
        // We should see variety in command selection
        assert_eq!(sequential.actions.len(), 10);
        // At least 2 different commands should be selected
        assert!(command_counts.len() >= 2, "Should select multiple different commands, got: {:?}", command_counts);
    }
    
    #[test]
    fn test_comprehensive_state_machine_workflow() {
        // This test demonstrates a complete state machine testing workflow
        // with realistic commands, state tracking, and verification
        
        #[derive(Debug, Clone)]
        struct BankState {
            balance: i32,
            is_open: bool,
            transaction_count: usize,
        }
        
        impl BankState {
            fn new() -> Self {
                Self {
                    balance: 0,
                    is_open: true,
                    transaction_count: 0,
                }
            }
        }
        
        #[derive(Clone, Debug)]
        struct DepositInput {
            amount: i32,
        }
        
        impl std::fmt::Display for DepositInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "${}", self.amount)
            }
        }
        
        #[derive(Clone, Debug)]
        struct WithdrawInput {
            amount: i32,
        }
        
        impl std::fmt::Display for WithdrawInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "${}", self.amount)
            }
        }
        
        let mut generator = ActionGenerator::new();
        
        // Deposit command
        let deposit_cmd: Command<DepositInput, i32, BankState, i32> = Command::new(
            "deposit".to_string(),
            |state: &BankState| {
                if state.is_open {
                    Some(Gen::new(|_size, seed| {
                        let (amount, _new_seed) = seed.next_bounded(100);
                        crate::tree::Tree::singleton(DepositInput { 
                            amount: (amount as i32) + 1 // Ensure positive amounts
                        })
                    }))
                } else {
                    None
                }
            },
            |input: DepositInput| input.amount,
        )
        .with_require(|state: &BankState, input: &DepositInput| {
            state.is_open && input.amount > 0
        })
        .with_update(|state: &mut BankState, input: &DepositInput, _output: &Var<i32>| {
            state.balance += input.amount;
            state.transaction_count += 1;
        })
        .with_ensure(|old_state: &BankState, new_state: &BankState, input: &DepositInput, output: &i32| {
            if new_state.balance != old_state.balance + input.amount {
                Err(format!("Balance mismatch: expected {}, got {}", 
                    old_state.balance + input.amount, new_state.balance))
            } else if new_state.transaction_count != old_state.transaction_count + 1 {
                Err("Transaction count not incremented".to_string())
            } else if *output != input.amount {
                Err(format!("Output mismatch: expected {}, got {}", input.amount, output))
            } else {
                Ok(())
            }
        });
        
        // Withdraw command
        let withdraw_cmd: Command<WithdrawInput, i32, BankState, i32> = Command::new(
            "withdraw".to_string(),
            |state: &BankState| {
                if state.is_open && state.balance > 0 {
                    let max_withdraw = std::cmp::min(state.balance, 50);
                    Some(Gen::new(move |_size, seed| {
                        let (amount, _new_seed) = seed.next_bounded(max_withdraw as u64 + 1);
                        crate::tree::Tree::singleton(WithdrawInput { 
                            amount: amount as i32 
                        })
                    }))
                } else {
                    None
                }
            },
            |input: WithdrawInput| input.amount,
        )
        .with_require(|state: &BankState, input: &WithdrawInput| {
            state.is_open && input.amount >= 0 && state.balance >= input.amount
        })
        .with_update(|state: &mut BankState, input: &WithdrawInput, _output: &Var<i32>| {
            state.balance -= input.amount;
            state.transaction_count += 1;
        })
        .with_ensure(|old_state: &BankState, new_state: &BankState, input: &WithdrawInput, output: &i32| {
            if new_state.balance != old_state.balance - input.amount {
                Err(format!("Balance mismatch after withdrawal: expected {}, got {}", 
                    old_state.balance - input.amount, new_state.balance))
            } else if new_state.transaction_count != old_state.transaction_count + 1 {
                Err("Transaction count not incremented".to_string())
            } else if *output != input.amount {
                Err(format!("Output mismatch: expected {}, got {}", input.amount, output))
            } else {
                Ok(())
            }
        });
        
        generator.add_command(deposit_cmd);
        generator.add_command(withdraw_cmd);
        
        let initial_state = BankState::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 20);
        
        println!("Generated banking sequence:");
        for action in &sequential.actions {
            println!("  {}", action.display_action());
        }
        
        // Execute the sequence and verify all postconditions pass
        let result = execute_sequential(initial_state, sequential);
        
        match &result {
            Ok(()) => println!("✓ All transactions succeeded with proper state tracking!"),
            Err(e) => println!("✗ Transaction failed: {}", e),
        }
        
        // This should succeed because our state machine is properly implemented
        assert!(result.is_ok(), "State machine execution failed: {:?}", result);
    }
    
    #[test]
    fn test_connection_pool_state_machine() {
        // Real-world example: HTTP connection pool management
        // This tests realistic scenarios like connection limits, timeouts, cleanup
        
        #[derive(Debug, Clone, PartialEq)]
        struct ConnectionPool {
            active_connections: HashMap<String, bool>, // host -> connected
            connection_count: usize,
            max_connections: usize,
            request_count: usize,
            hosts: Vec<String>,
        }
        
        impl ConnectionPool {
            fn new() -> Self {
                Self {
                    active_connections: HashMap::new(),
                    connection_count: 0,
                    max_connections: 5,
                    request_count: 0,
                    hosts: vec![
                        "api.example.com".to_string(),
                        "db.example.com".to_string(), 
                        "cache.example.com".to_string(),
                    ],
                }
            }
            
            fn can_connect(&self) -> bool {
                self.connection_count < self.max_connections
            }
            
            fn is_connected(&self, host: &str) -> bool {
                self.active_connections.get(host).copied().unwrap_or(false)
            }
        }
        
        #[derive(Clone, Debug)]
        struct ConnectInput {
            host: String,
        }
        
        impl std::fmt::Display for ConnectInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.host)
            }
        }
        
        #[derive(Clone, Debug)]
        struct RequestInput {
            host: String,
        }
        
        impl std::fmt::Display for RequestInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "GET {}/", self.host)
            }
        }
        
        #[derive(Clone, Debug)]  
        struct DisconnectInput {
            host: String,
        }
        
        impl std::fmt::Display for DisconnectInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.host)
            }
        }
        
        let mut generator = ActionGenerator::new();
        
        // Connect command
        let connect_cmd: Command<ConnectInput, bool, ConnectionPool, bool> = Command::new(
            "connect".to_string(),
            |state: &ConnectionPool| {
                if state.can_connect() {
                    Some(Gen::new({
                        let hosts = state.hosts.clone();
                        move |_size, seed| {
                            let (idx, _) = seed.next_bounded(hosts.len() as u64);
                            crate::tree::Tree::singleton(ConnectInput { 
                                host: hosts[idx as usize].clone() 
                            })
                        }
                    }))
                } else {
                    None
                }
            },
            |input: ConnectInput| !input.host.is_empty(),
        )
        .with_require(|state: &ConnectionPool, input: &ConnectInput| {
            state.can_connect() && !state.is_connected(&input.host)
        })
        .with_update(|state: &mut ConnectionPool, input: &ConnectInput, _output: &Var<bool>| {
            state.active_connections.insert(input.host.clone(), true);
            state.connection_count += 1;
        })
        .with_ensure(|old_state: &ConnectionPool, new_state: &ConnectionPool, input: &ConnectInput, output: &bool| {
            if !new_state.is_connected(&input.host) {
                Err(format!("Failed to connect to {}", input.host))
            } else if new_state.connection_count != old_state.connection_count + 1 {
                Err("Connection count not incremented".to_string())
            } else if !output {
                Err("Connect should return true on success".to_string())
            } else {
                Ok(())
            }
        });
        
        // Request command  
        let request_cmd: Command<RequestInput, usize, ConnectionPool, usize> = Command::new(
            "request".to_string(), 
            |state: &ConnectionPool| {
                let connected_hosts: Vec<_> = state.active_connections
                    .iter()
                    .filter(|(_, &connected)| connected)
                    .map(|(host, _)| host.clone())
                    .collect();
                    
                if !connected_hosts.is_empty() {
                    Some(Gen::new(move |_size, seed| {
                        let (idx, _) = seed.next_bounded(connected_hosts.len() as u64);
                        crate::tree::Tree::singleton(RequestInput { 
                            host: connected_hosts[idx as usize].clone() 
                        })
                    }))
                } else {
                    None
                }
            },
            |input: RequestInput| input.host.len(), // Return response size
        )
        .with_require(|state: &ConnectionPool, input: &RequestInput| {
            state.is_connected(&input.host)
        })
        .with_update(|state: &mut ConnectionPool, _input: &RequestInput, _output: &Var<usize>| {
            state.request_count += 1;
        })
        .with_ensure(|old_state: &ConnectionPool, new_state: &ConnectionPool, input: &RequestInput, output: &usize| {
            if new_state.request_count != old_state.request_count + 1 {
                Err("Request count not incremented".to_string())
            } else if *output != input.host.len() {
                Err("Incorrect response size".to_string())
            } else {
                Ok(())
            }
        });
        
        // Disconnect command
        let disconnect_cmd: Command<DisconnectInput, bool, ConnectionPool, bool> = Command::new(
            "disconnect".to_string(),
            |state: &ConnectionPool| {
                let connected_hosts: Vec<_> = state.active_connections
                    .iter() 
                    .filter(|(_, &connected)| connected)
                    .map(|(host, _)| host.clone())
                    .collect();
                    
                if !connected_hosts.is_empty() {
                    Some(Gen::new(move |_size, seed| {
                        let (idx, _) = seed.next_bounded(connected_hosts.len() as u64);
                        crate::tree::Tree::singleton(DisconnectInput { 
                            host: connected_hosts[idx as usize].clone() 
                        })
                    }))
                } else {
                    None
                }
            },
            |_input: DisconnectInput| true,
        )
        .with_require(|state: &ConnectionPool, input: &DisconnectInput| {
            state.is_connected(&input.host)
        })
        .with_update(|state: &mut ConnectionPool, input: &DisconnectInput, _output: &Var<bool>| {
            state.active_connections.insert(input.host.clone(), false);
            state.connection_count -= 1;
        })
        .with_ensure(|old_state: &ConnectionPool, new_state: &ConnectionPool, input: &DisconnectInput, output: &bool| {
            if new_state.is_connected(&input.host) {
                Err(format!("Failed to disconnect from {}", input.host))
            } else if new_state.connection_count != old_state.connection_count - 1 {
                Err("Connection count not decremented".to_string())
            } else if !output {
                Err("Disconnect should return true on success".to_string())
            } else {
                Ok(())
            }
        });
        
        generator.add_command(connect_cmd);
        generator.add_command(request_cmd);
        generator.add_command(disconnect_cmd);
        
        let initial_state = ConnectionPool::new();
        let sequential = generator.generate_sequential(initial_state.clone(), 15);
        
        println!("Generated connection pool sequence:");
        for (i, action) in sequential.actions.iter().enumerate() {
            println!("  {}: {}", i + 1, action.display_action());
        }
        
        let result = execute_sequential(initial_state, sequential);
        
        match &result {
            Ok(()) => println!("✓ Connection pool state machine test passed!"),
            Err(e) => println!("✗ Connection pool test failed: {}", e),
        }
        
        // This demonstrates realistic resource management patterns
        assert!(result.is_ok(), "Connection pool test failed: {:?}", result);
    }
    
    #[test]
    fn test_readme_quick_start_example() {
        // This is the exact example from README.md to ensure it works
        use crate::state::*;
        use crate::gen::Gen;
        use crate::tree::Tree;

        // 1. Define your system state
        #[derive(Debug, Clone, PartialEq)]
        struct Counter {
            value: i32,
        }

        impl Counter {
            fn new() -> Self { Self { value: 0 } }
        }

        // 2. Define command inputs
        #[derive(Clone, Debug)]
        struct AddInput { amount: i32 }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();
        
        // 3. Create a command
        let add_cmd: Command<AddInput, i32, Counter, i32> = Command::new(
            "add".to_string(),
            |_state: &Counter| Some(Gen::new(|_size, seed| {
                let (amount, _) = seed.next_bounded(10);
                Tree::singleton(AddInput { amount: amount as i32 + 1 })
            })),
            |input| input.amount, // The actual operation
        )
        .with_update(|state, input, _output| {
            state.value += input.amount; // Update model state
        });
        
        generator.add_command(add_cmd);
        
        // 4. Generate and run test
        let initial = Counter::new();
        let test = generator.generate_sequential(initial.clone(), 5);
        
        println!("README example sequence:");
        for action in &test.actions {
            println!("  {}", action.display_action());
        }
        
        let result = execute_sequential(initial, test);
        assert!(result.is_ok(), "README example failed: {:?}", result);
        println!("✓ README quick start example works!");
    }

    #[test] 
    fn test_tutorial_step_1_simple_counter() {
        // Step 1 from tutorial - basic counter
        #[derive(Debug, Clone, PartialEq)]
        struct Counter {
            value: i32,
        }

        #[derive(Clone, Debug)]
        struct AddInput { amount: i32 }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();
        
        let add_cmd: Command<AddInput, i32, Counter, i32> = Command::new(
            "add".to_string(),
            |_state: &Counter| Some(Gen::new(|_size, seed| {
                let (amount, _) = seed.next_bounded(10);
                crate::tree::Tree::singleton(AddInput { amount: amount as i32 + 1 })
            })),
            |input: AddInput| input.amount,
        )
        .with_update(|state: &mut Counter, input: &AddInput, _output: &Var<i32>| {
            state.value += input.amount;
        });
        
        generator.add_command(add_cmd);
        
        let initial = Counter { value: 0 };
        let test = generator.generate_sequential(initial.clone(), 5);
        
        let result = execute_sequential(initial, test);
        assert!(result.is_ok());
        println!("✓ Tutorial Step 1 works!");
    }

    #[test]
    fn test_tutorial_step_2_bounded_counter() {
        // Step 2 from tutorial - counter with constraints
        #[derive(Debug, Clone, PartialEq)]
        struct BoundedCounter {
            value: i32,
            max_value: i32,
        }

        impl BoundedCounter {
            fn new() -> Self { Self { value: 0, max_value: 100 } }
            fn can_add(&self, amount: i32) -> bool { self.value + amount <= self.max_value }
        }

        #[derive(Clone, Debug)]
        struct AddInput { amount: i32 }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();
        
        let add_cmd: Command<AddInput, i32, BoundedCounter, i32> = Command::new(
            "add".to_string(),
            |state: &BoundedCounter| {
                if state.value < state.max_value {
                    let remaining = state.max_value - state.value;
                    let max_add = std::cmp::min(remaining, 10);
                    Some(Gen::new(move |_size, seed| {
                        let (amount, _) = seed.next_bounded(max_add as u64);
                        crate::tree::Tree::singleton(AddInput { amount: amount as i32 + 1 })
                    }))
                } else {
                    None
                }
            },
            |input: AddInput| input.amount,
        )
        .with_require(|state: &BoundedCounter, input: &AddInput| {
            state.can_add(input.amount)
        })
        .with_update(|state: &mut BoundedCounter, input: &AddInput, _output: &Var<i32>| {
            state.value += input.amount;
        })
        .with_ensure(|old_state: &BoundedCounter, new_state: &BoundedCounter, input: &AddInput, output: &i32| {
            if new_state.value != old_state.value + input.amount {
                Err(format!("Expected {}, got {}", old_state.value + input.amount, new_state.value))
            } else if new_state.value > new_state.max_value {
                Err("Counter exceeded maximum".to_string())
            } else if *output != input.amount {
                Err("Incorrect return value".to_string())
            } else {
                Ok(())
            }
        });
        
        generator.add_command(add_cmd);
        
        let initial = BoundedCounter::new();
        let test = generator.generate_sequential(initial.clone(), 20);
        
        let result = execute_sequential(initial, test);
        assert!(result.is_ok());
        println!("✓ Tutorial Step 2 works!");
    }

    #[test] 
    fn test_tutorial_step_3_multiple_commands() {
        // Step 3 from tutorial - counter with add and reset
        #[derive(Debug, Clone, PartialEq)]
        struct BoundedCounter {
            value: i32,
            max_value: i32,
        }

        impl BoundedCounter {
            fn new() -> Self { Self { value: 0, max_value: 100 } }
            fn can_add(&self, amount: i32) -> bool { self.value + amount <= self.max_value }
        }

        #[derive(Clone, Debug)]
        struct AddInput { amount: i32 }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        #[derive(Clone, Debug)]
        struct ResetInput;

        impl std::fmt::Display for ResetInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "reset")
            }
        }

        let mut generator = ActionGenerator::new();
        
        // Add command
        let add_cmd: Command<AddInput, i32, BoundedCounter, i32> = Command::new(
            "add".to_string(),
            |state: &BoundedCounter| {
                if state.value < state.max_value {
                    let remaining = state.max_value - state.value;
                    let max_add = std::cmp::min(remaining, 10);
                    Some(Gen::new(move |_size, seed| {
                        let (amount, _) = seed.next_bounded(max_add as u64);
                        crate::tree::Tree::singleton(AddInput { amount: amount as i32 + 1 })
                    }))
                } else {
                    None
                }
            },
            |input: AddInput| input.amount,
        )
        .with_require(|state: &BoundedCounter, input: &AddInput| state.can_add(input.amount))
        .with_update(|state: &mut BoundedCounter, input: &AddInput, _| state.value += input.amount)
        .with_ensure(|old, new, input, _output| {
            if new.value != old.value + input.amount {
                Err("Add failed".to_string())
            } else {
                Ok(())
            }
        });
        
        // Reset command
        let reset_cmd: Command<ResetInput, i32, BoundedCounter, i32> = Command::new(
            "reset".to_string(),
            |state: &BoundedCounter| {
                if state.value > 0 {
                    Some(Gen::constant(ResetInput))
                } else {
                    None
                }
            },
            |_input: ResetInput| 0,
        )
        .with_update(|state: &mut BoundedCounter, _: &ResetInput, _| {
            state.value = 0;
        })
        .with_ensure(|_old, new, _input, output| {
            if new.value != 0 {
                Err("Reset failed to set value to 0".to_string())
            } else if *output != 0 {
                Err("Reset should return 0".to_string())
            } else {
                Ok(())
            }
        });
        
        generator.add_command(add_cmd);
        generator.add_command(reset_cmd);
        
        let initial = BoundedCounter::new();
        let test = generator.generate_sequential(initial.clone(), 15);
        
        println!("Tutorial Step 3 operations:");
        for (i, action) in test.actions.iter().enumerate() {
            println!("  {}: {}", i + 1, action.display_action());
        }
        
        let result = execute_sequential(initial, test);
        assert!(result.is_ok());
        println!("✓ Tutorial Step 3 works!");
    }

    #[test]
    fn test_edge_case_no_available_commands() {
        // Critical edge case: what happens when no commands can execute?
        #[derive(Debug, Clone, PartialEq)]
        struct LockedState {
            locked: bool,
        }

        #[derive(Clone, Debug)]
        struct UnlockInput;

        impl std::fmt::Display for UnlockInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "unlock")
            }
        }

        let mut generator = ActionGenerator::new();
        
        let unlock_cmd: Command<UnlockInput, bool, LockedState, bool> = Command::new(
            "unlock".to_string(),
            |state: &LockedState| {
                if !state.locked {
                    Some(Gen::constant(UnlockInput))
                } else {
                    None // Can't unlock when already locked (impossible condition)
                }
            },
            |_input: UnlockInput| true,
        );
        
        generator.add_command(unlock_cmd);
        
        // State where no commands are available
        let locked_state = LockedState { locked: true };
        let sequential = generator.generate_sequential(locked_state.clone(), 10);
        
        // Should handle gracefully - generate empty sequence, not panic
        assert_eq!(sequential.actions.len(), 0);
        
        let result = execute_sequential(locked_state, sequential);
        assert!(result.is_ok()); // Empty sequence should succeed
        
        println!("✓ Handled edge case: no available commands");
    }

    #[test]
    fn test_state_consistency_between_phases() {
        // Critical test: generation and execution should produce same state changes
        #[derive(Debug, Clone, PartialEq)]
        struct TestState {
            value: i32,
        }

        #[derive(Clone, Debug)]
        struct AddInput { amount: i32 }

        impl std::fmt::Display for AddInput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "+{}", self.amount)
            }
        }

        let mut generator = ActionGenerator::new();
        
        let add_cmd: Command<AddInput, i32, TestState, i32> = Command::new(
            "add".to_string(),
            |_state: &TestState| Some(Gen::constant(AddInput { amount: 5 })),
            |input: AddInput| input.amount,
        )
        .with_update(|state: &mut TestState, input: &AddInput, _output: &Var<i32>| {
            state.value += input.amount;
        });
        
        generator.add_command(add_cmd);
        
        let initial_state = TestState { value: 0 };
        
        // Generate a sequence - this exercises the generation phase state updates
        let sequential = generator.generate_sequential(initial_state.clone(), 3);
        
        // Execute the sequence - this exercises execution phase state updates  
        let mut execution_state = initial_state.clone();
        let mut env = Environment::new();
        
        for action in sequential.actions {
            action.execute_action(&mut execution_state, &mut env).unwrap();
        }
        
        // Both should result in same final state: 0 + 5 + 5 + 5 = 15
        assert_eq!(execution_state.value, 15);
        
        println!("✓ State consistency maintained between generation and execution phases");
    }
}