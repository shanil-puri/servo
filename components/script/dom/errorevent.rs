/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, ErrorEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use js::jsapi::JSContext;
use dom::bindings::trace::JSTraceable;

use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventTypeId, ErrorEventTypeId};
use servo_util::str::DOMString;

use dom::bindings::cell::DOMRefCell;
use std::cell::{Cell};
use js::jsval::{JSVal, NullValue};

#[dom_struct]
pub struct ErrorEvent {
    event: Event,
    message: DOMRefCell<DOMString>,
    filename: DOMRefCell<DOMString>,
    lineno: Cell<u32>,
    colno: Cell<u32>,
    error: Cell<JSVal>
}

impl ErrorEventDerived for Event {
    fn is_errorevent(&self) -> bool {
        *self.type_id() == ErrorEventTypeId
    }
}

impl ErrorEvent {
    fn new_inherited(type_id: EventTypeId) -> ErrorEvent {
        ErrorEvent {
            event: Event::new_inherited(type_id),
            message: DOMRefCell::new("".to_string()),
            filename: DOMRefCell::new("".to_string()),
            lineno: Cell::new(0),
            colno: Cell::new(0),
            error: Cell::new(NullValue())
        }
    }

    pub fn new_uninitialized(global: &GlobalRef) -> Temporary<ErrorEvent> {
        reflect_dom_object(box ErrorEvent::new_inherited(ErrorEventTypeId),
                           *global,
                           ErrorEventBinding::Wrap)
    }

    pub fn new(global: &GlobalRef,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               message: DOMString,
               filename: DOMString,
               lineno: u32,
               colno: u32,
               error: JSVal) -> Temporary<ErrorEvent> {
        let ev = ErrorEvent::new_uninitialized(&*global).root();
        let event: JSRef<Event> = EventCast::from_ref(*ev);
        event.InitEvent(type_, can_bubble, cancelable);
        *ev.message.borrow_mut() = message;
        *ev.filename.borrow_mut() = filename;
        ev.lineno.set(lineno);
        ev.colno.set(colno);
        ev.error.set(error);
        Temporary::from_rooted(*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &ErrorEventBinding::ErrorEventInit) -> Fallible<Temporary<ErrorEvent>>{
        let msg = match init.message.as_ref() {
            Some(message) => message.clone(),
            None => "".to_string(),
        };

        let file_name = match init.filename.as_ref() {
            None => "".into_string(),
            Some(filename) => filename.clone(),
        };

        let line_num = init.lineno.unwrap_or(0);

        let col_num = init.colno.unwrap_or(0);

        let event = ErrorEvent::new(&*global, type_,
                                init.parent.bubbles, init.parent.cancelable,
                                msg, file_name,
                                line_num, col_num, init.error);
        Ok(event)
    }

}

impl<'a> ErrorEventMethods for JSRef<'a, ErrorEvent> {
    fn Lineno(self) -> u32 {
        self.lineno.get()
    }

    fn Colno(self) -> u32 {
        self.colno.get()
    }

    fn Message(self) -> DOMString {
        self.message.borrow().clone()
    }

    fn Filename(self) -> DOMString {
        self.filename.borrow().clone()
    }

    fn Error(self, _cx: *mut JSContext) -> JSVal {
        self.error.get()
        propagate_error(&self, Error);
    }

}

impl Reflectable for ErrorEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }
}

fn propagate_error(global: &GlobalRef, error: JSVal) {
    match msg_type {
        LogMsg => {
            let pipelineId = global.as_window().page().id;
            global.as_window().page().devtools_chan.as_ref().map(|chan| {
                chan.send(SendConsoleMessage(pipelineId, LogMessage(message.clone())));
            });
        }

        WarnMsg => {
            //TODO: to be implemented for warning messages
        }
    }
}