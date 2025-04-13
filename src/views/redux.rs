use crate::*;

pub fn redux<
    V: View,
    A: 'static,
    A2: 'static,
    S: 'static,
    D: Fn() -> S + Clone + 'static,
    F: Fn(&S) -> V + Clone + 'static,
    R: Fn(&mut S, &A) -> A2 + 'static + Clone,
>(
    initial: D,
    reducer: R,
    f: F,
) -> impl View {
    state(initial, move |state_handle, ctx| {
        let r = reducer.clone();
        f(&ctx[state_handle]).handle(move |ctx, action: &A| r(&mut ctx[state_handle], action))
    })
}
