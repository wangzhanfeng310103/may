use std::thread;
use generator::{co_yield_with, co_get_yield};
use coroutine_impl::{is_coroutine, current_cancel_data};
use coroutine_impl::{CoroutineImpl, EventResult, EventSource, EventSubscriber};
use scheduler::get_scheduler;

struct Yield {
}

impl EventSource for Yield {
    fn subscribe(&mut self, co: CoroutineImpl) {
        // just re-push the coroutine to the ready list
        get_scheduler().schedule(co);
        // println!("yield_out()");
    }
}

/// yield internal `EventSource` ref
/// it's ok to return a ref of object on the generator's stack
/// just like return the ref of a struct member
#[inline]
pub fn yield_with<T: EventSource>(resource: &T) {
    let cancel = current_cancel_data();
    // if cancel detected in user space
    // no need to get into kernel any more
    if cancel.is_canceled() {
        return resource.yield_back(cancel);
    }

    let r = resource as &EventSource as *const _ as *mut _;
    let es = EventSubscriber::new(r);
    co_yield_with(es);

    resource.yield_back(cancel);
    cancel.clear();
}

/// set the coroutine para that passed into it
#[inline]
pub fn set_co_para(co: &mut CoroutineImpl, v: EventResult) {
    co.set_para(v);
}

/// get the coroutine para from the coroutine context
#[inline]
pub fn get_co_para() -> Option<EventResult> {
    co_get_yield::<EventResult>()
}

#[inline]
pub fn yield_now() {
    if !is_coroutine() {
        return thread::yield_now();
    }
    let y = Yield {};
    // it's safe to use the stack value here
    yield_with(&y);
}
