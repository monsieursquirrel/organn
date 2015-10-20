extern crate libc;
extern crate CoreFoundation_sys;
extern crate midi;

mod core_midi_services;

use std::ptr;
use std::ffi::CString;
use std::marker::PhantomData;

pub struct MidiWrap<A> where A: Fn(midi::Message) {
    client: core_midi_services::MIDIClientRef,
    port: core_midi_services::MIDIPortRef,
    thing: Box<A>
}

impl<A> MidiWrap<A> where A: Fn(midi::Message)  {
    pub fn new(clinet_name: &str, port_name: &str, callback: A) -> Option<MidiWrap<A>> {

        let thing = Box::new(callback);

        // create a midi client
        let mut client: core_midi_services::MIDIClientRef = 0;
        let status;
        unsafe {
            let funky_string = CoreFoundation_sys::CFStringCreateWithCString(
                CoreFoundation_sys::kCFAllocatorMalloc,
                CString::new(clinet_name).unwrap().as_ptr(),
                CoreFoundation_sys::kCFStringEncodingUTF8);
            status = core_midi_services::MIDIClientCreate(funky_string, None, ptr::null(), &mut client);
        }
        if status != 0 {
            return None;
        }

        // create an input port with a callback
        let mut port: core_midi_services::MIDIPortRef = 0;
        let status;
        unsafe {
            use std::intrinsics::transmute;
            let funky_string = CoreFoundation_sys::CFStringCreateWithCString(
                CoreFoundation_sys::kCFAllocatorMalloc,
                CString::new(port_name).unwrap().as_ptr(),
                CoreFoundation_sys::kCFStringEncodingUTF8);
            status = core_midi_services::MIDIInputPortCreate(client,funky_string, Some(MidiWrap::<A>::midi_callback), transmute(&*thing), &mut port);

            // connect everything to the input
            let num_sources = core_midi_services::MIDIGetNumberOfSources();

            for i in (0..num_sources) {
                core_midi_services::MIDIPortConnectSource(port, core_midi_services::MIDIGetSource(i), ptr::null_mut());
            }
        }
        if status != 0 {
            return None;
        }

        Some(MidiWrap {
            client: client,
            port: port,
            thing: thing
        })
    }

    extern "C" fn midi_callback(pktlist: *const core_midi_services::MIDIPacketList,
                                read_proc_ref_con: *mut ::libc::c_void,
                                _: *mut ::libc::c_void) -> () {
        unsafe {
            use std::intrinsics::transmute;
            use std::slice;

            let wrap_fn: &A = transmute(read_proc_ref_con);
            let mut packet = &(*pktlist).packet[0];
            for _ in (0..(*pktlist).numPackets) {
                let bytes = slice::from_raw_parts(packet.data.as_ptr(), packet.length  as usize);
                if let Some(message) = parse_midi_bytes(bytes) {
                    wrap_fn(message);
                }

                packet = core_midi_services::MIDIPacketNext(packet);
            }
        }
    }
}

impl<A> Drop for MidiWrap<A> where A: Fn(midi::Message)  {
    fn drop(&mut self) {
        unsafe {
            core_midi_services::MIDIPortDispose(self.port);
            core_midi_services::MIDIClientDispose(self.client);
        }
    }
}

fn parse_midi_bytes(bytes: &[u8]) -> Option<midi::Message> {
    if bytes.len() >= 1 {
        let (status, channel) = midi::utils::from_status_byte(bytes[0]);
        match status {
            8 if bytes.len() >= 3 => { Some(midi::NoteOff(channel, bytes[1], bytes[2])) }
            9 if bytes.len() >= 3 => { Some(midi::NoteOn(channel, bytes[1], bytes[2])) }
            _ => { None }
        }
    }
    else {
        None
    }
}
