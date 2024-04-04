use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

use once_cell::sync::OnceCell;

use crate::frame::{AsciiFrame, Frame, Full};

pub static INPUT_FRAME_QUEUE_INSTANCE: OnceCell<GenericSharedQueue<Frame>> = OnceCell::new();
pub static ASCII_FRAME_QUEUE_INSTANCE: OnceCell<GenericSharedQueue<AsciiFrame<Full>>> =
    OnceCell::new();
pub static OUTPUT_FRAME_QUEUE_INSTANCE: OnceCell<GenericSharedQueue<Frame>> = OnceCell::new();

pub struct GenericSharedQueue<T>
where
    T: Queueable,
{
    pub queue: Mutex<VecDeque<T>>,
    pub condvar: Condvar,
}

impl<T> GenericSharedQueue<T>
where
    T: Queueable,
{
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }

    /// retrieve the global shared queue for the type of frame
    pub fn global(frame_type: FrameType) -> &'static GenericSharedQueue<T> {
        T::queue_instance(frame_type)
            .get()
            .expect("Queue is not initialized")
    }
}

pub trait Queueable {
    fn queue_instance(frame_type: FrameType) -> &'static OnceCell<GenericSharedQueue<Self>>
    where
        Self: Sized;
}

pub enum FrameType {
    Input,
    Output,
}

impl Queueable for Frame {
    fn queue_instance(frame_type: FrameType) -> &'static OnceCell<GenericSharedQueue<Self>> {
        match frame_type {
            FrameType::Input => &INPUT_FRAME_QUEUE_INSTANCE,
            FrameType::Output => &OUTPUT_FRAME_QUEUE_INSTANCE,
        }
    }
}

impl Queueable for AsciiFrame<Full> {
    fn queue_instance(frame_type: FrameType) -> &'static OnceCell<GenericSharedQueue<Self>>
where {
        match frame_type {
            _ => &ASCII_FRAME_QUEUE_INSTANCE,
        }
    }
}
