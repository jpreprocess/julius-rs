use std::{
    ffi::{c_void, CString},
    ptr::null_mut,
};

use adin::ADIn;
use iter::BindIterator;
use recog_process::RecogProcess;
use strum_macros::FromRepr;

mod iter;

pub mod adin;
pub mod recog_process;
pub mod sentence_align;

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum CallbackType {
    Poll = libjulius_sys::CALLBACK_POLL,
    EventProcessOnline = libjulius_sys::CALLBACK_EVENT_PROCESS_ONLINE,
    EventProcessOffline = libjulius_sys::CALLBACK_EVENT_PROCESS_OFFLINE,
    EventStreamBegin = libjulius_sys::CALLBACK_EVENT_STREAM_BEGIN,
    EventStreamEnd = libjulius_sys::CALLBACK_EVENT_STREAM_END,
    EventSpeechReady = libjulius_sys::CALLBACK_EVENT_SPEECH_READY,
    EventSpeechStart = libjulius_sys::CALLBACK_EVENT_SPEECH_START,
    EventSpeechStop = libjulius_sys::CALLBACK_EVENT_SPEECH_STOP,
    EventRecognitionBegin = libjulius_sys::CALLBACK_EVENT_RECOGNITION_BEGIN,
    EventRecognitionEnd = libjulius_sys::CALLBACK_EVENT_RECOGNITION_END,
    EventSegmentBegin = libjulius_sys::CALLBACK_EVENT_SEGMENT_BEGIN,
    EventSegmentEnd = libjulius_sys::CALLBACK_EVENT_SEGMENT_END,
    EventPass1Begin = libjulius_sys::CALLBACK_EVENT_PASS1_BEGIN,
    EventPass1Frame = libjulius_sys::CALLBACK_EVENT_PASS1_FRAME,
    EventPass1End = libjulius_sys::CALLBACK_EVENT_PASS1_END,
    ResultPass1Interim = libjulius_sys::CALLBACK_RESULT_PASS1_INTERIM,
    ResultPass1 = libjulius_sys::CALLBACK_RESULT_PASS1,
    ResultPass1Graph = libjulius_sys::CALLBACK_RESULT_PASS1_GRAPH,
    StatusParam = libjulius_sys::CALLBACK_STATUS_PARAM,
    EventPass2Begin = libjulius_sys::CALLBACK_EVENT_PASS2_BEGIN,
    EventPass2End = libjulius_sys::CALLBACK_EVENT_PASS2_END,
    Result = libjulius_sys::CALLBACK_RESULT,
    ResultGmm = libjulius_sys::CALLBACK_RESULT_GMM,
    ResultGraph = libjulius_sys::CALLBACK_RESULT_GRAPH,
    ResultConfnet = libjulius_sys::CALLBACK_RESULT_CONFNET,
    EventPause = libjulius_sys::CALLBACK_EVENT_PAUSE,
    EventResume = libjulius_sys::CALLBACK_EVENT_RESUME,
    PauseFunction = libjulius_sys::CALLBACK_PAUSE_FUNCTION,
    DebugPass2Pop = libjulius_sys::CALLBACK_DEBUG_PASS2_POP,
    DebugPass2Push = libjulius_sys::CALLBACK_DEBUG_PASS2_PUSH,
    ResultPass1Determined = libjulius_sys::CALLBACK_RESULT_PASS1_DETERMINED,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum AdinCallbackType {
    Captured = libjulius_sys::CALLBACK_ADIN_CAPTURED,
    Triggered = libjulius_sys::CALLBACK_ADIN_TRIGGERED,
}

#[derive(Debug)]
pub struct JConf<'a>(&'a mut libjulius_sys::Jconf);
impl<'a> JConf<'a> {
    pub fn new() -> Result<Self, anyhow::Error> {
        let jconf = unsafe { libjulius_sys::j_jconf_new() };
        if jconf.is_null() {
            Err(anyhow::anyhow!("JConf failed"))
        } else {
            Ok(Self(unsafe { &mut *jconf }))
        }
    }
    pub fn from_string(string: &str) -> Result<Self, anyhow::Error> {
        let cstr = CString::new(string)?;
        let jconf = unsafe { libjulius_sys::j_config_load_string_new(cstr.into_raw()) };
        if jconf.is_null() {
            Err(anyhow::anyhow!("JConf failed"))
        } else {
            Ok(Self(unsafe { &mut *jconf }))
        }
    }

    pub unsafe fn as_raw_ptr(&self) -> *mut libjulius_sys::Jconf {
        self.0 as *const libjulius_sys::Jconf as *mut libjulius_sys::Jconf
    }
}

impl<'a> Drop for JConf<'a> {
    fn drop(&mut self) {
        unsafe {
            libjulius_sys::j_jconf_free(&mut *self.0);
        }
    }
}

#[derive(Debug)]
pub struct Recog<'a>(&'a mut libjulius_sys::Recog);
impl<'a> Recog<'a> {
    pub fn from_jconf(jconf: JConf) -> Result<Self, anyhow::Error> {
        let recog = unsafe { libjulius_sys::j_create_instance_from_jconf(&mut *jconf.0) };
        std::mem::forget(jconf);
        if recog.is_null() {
            Err(anyhow::anyhow!("Recog failed"))
        } else {
            Ok(Self(unsafe { &mut *recog }))
        }
    }

    pub fn get_processes(&self) -> BindIterator<libjulius_sys::RecogProcess, RecogProcess> {
        let first_process = self.0.process_list;
        BindIterator::new(
            first_process,
            Box::new(|curr| unsafe { (*curr).next }),
            Box::new(|curr| unsafe { RecogProcess::new(*curr) }),
        )
    }

    pub fn adin_init(&mut self) -> Result<(), anyhow::Error> {
        let ret = unsafe { libjulius_sys::j_adin_init(&mut *self.0) };
        match ret {
            1 => Ok(()),
            0 => Err(anyhow::anyhow!("Failed to initialize input device")),
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_adin_mut(&mut self) -> &mut ADIn {
        unsafe { &mut *(self.0.adin as *mut ADIn) }
    }
    /// Setup custom ADIn.
    ///
    /// The following members of ADIn will interfere with custom ADIn.
    /// - ad_standby
    /// - ad_begin
    /// - ad_end
    /// - ad_resume
    /// - ad_pause
    /// - ad_terminate
    /// - ad_read
    /// - ad_input_name
    /// - silence_cut_default
    /// - enable_thread
    /// - down_sample
    ///
    pub fn custom_adin<T: FnMut(usize) -> Option<U> + 'a, U: AsRef<[i16]>>(&mut self, ad_read: T) {
        self.get_adin_mut().ad_read_inject_prepare();
        self.add_callback_adin_inject(ad_read);
    }

    pub fn open_stream(&mut self, file_or_dev_name: Option<&str>) -> Result<(), anyhow::Error> {
        let cstr = match file_or_dev_name {
            Some(fv) => Some(CString::new(fv)?),
            None => None,
        };
        let ret = unsafe {
            libjulius_sys::j_open_stream(
                &mut *self.0,
                match cstr {
                    Some(fv) => fv.into_raw(),
                    None => null_mut(),
                },
            )
        };
        match ret {
            0 => Ok(()),
            -1 => Err(anyhow::anyhow!("Error in input stream")),
            -2 => Err(anyhow::anyhow!("Failed to begin input stream")),
            _ => unreachable!(),
        }
    }

    pub fn recognize_stream(&mut self) -> Result<(), anyhow::Error> {
        let ret = unsafe { libjulius_sys::j_recognize_stream(&mut *self.0) };
        match ret {
            0 => Ok(()),
            ret => Err(anyhow::anyhow!(ret)),
        }
    }

    pub fn close_stream(&mut self) -> Result<(), anyhow::Error> {
        let ret = unsafe { libjulius_sys::j_close_stream(&mut *self.0) };
        match ret {
            0 => Ok(()),
            ret => Err(anyhow::anyhow!(ret)),
        }
    }

    pub fn add_callback<T: FnMut(&mut Self) + 'a>(&mut self, cb_type: CallbackType, callback: T) {
        let code = cb_type as i32;
        let cb = Box::new(Box::new(callback));
        unsafe {
            libjulius_sys::callback_add(
                &mut *self.0,
                code,
                Some(Self::cb::<T>),
                Box::into_raw(cb) as *mut _,
            );
        }
    }
    unsafe extern "C" fn cb<Env: Sized + FnMut(&mut Self)>(
        recog: *mut libjulius_sys::Recog,
        data: *mut c_void,
    ) {
        let closure: &mut Box<Env> = std::mem::transmute(data);
        let mut recog_wrapped = Self(&mut *recog);
        closure(&mut recog_wrapped);
        std::mem::forget(recog_wrapped);
    }

    pub fn add_callback_adin<T: FnMut(&mut Self, &[i16]) + 'a>(
        &mut self,
        cb_type: AdinCallbackType,
        callback: T,
    ) {
        let code = cb_type as i32;
        let cb = Box::new(Box::new(callback));
        unsafe {
            libjulius_sys::callback_add_adin(
                &mut *self.0,
                code,
                Some(Self::adin_cb::<T>),
                Box::into_raw(cb) as *mut _,
            );
        }
    }
    unsafe extern "C" fn adin_cb<Env: Sized + FnMut(&mut Self, &[i16])>(
        recog: *mut libjulius_sys::Recog,
        buf: *mut libjulius_sys::SP16,
        len: i32,
        data: *mut c_void,
    ) {
        let closure: &mut Box<Env> = std::mem::transmute(data);
        let buffer = std::slice::from_raw_parts(buf, len as usize);
        let mut recog_wrapped = Self(&mut *recog);
        closure(&mut recog_wrapped, buffer);
        std::mem::forget(recog_wrapped);
    }

    fn add_callback_adin_inject<T: FnMut(usize) -> Option<U> + 'a, U: AsRef<[i16]>>(
        &mut self,
        callback: T,
    ) {
        let cb = Box::new(Box::new(callback));
        unsafe {
            libjulius_sys::callback_add_adin(
                &mut *self.0,
                libjulius_sys::CALLBACK_ADIN_INJECT as i32,
                Some(Self::adin_inject_cb::<T, U>),
                Box::into_raw(cb) as *mut _,
            );
        }
    }
    unsafe extern "C" fn adin_inject_cb<
        Env: FnMut(usize) -> Option<Result> + 'a,
        Result: AsRef<[i16]>,
    >(
        recog: *mut libjulius_sys::Recog,
        // This is actually the pointer to cnt
        buf: *mut libjulius_sys::SP16,
        _len: i32,
        data: *mut c_void,
    ) {
        let ad_read: &mut Box<Env> = std::mem::transmute(data);
        let mut recog_wrapped = Self(&mut *recog);
        let cnt: &mut i32 = std::mem::transmute(buf);

        let adin = recog_wrapped.get_adin_mut();
        let data = ad_read(adin.samp_num() as usize);
        *cnt = adin.ad_read_inject_callback(data);

        std::mem::forget(recog_wrapped);
    }

    pub unsafe fn as_raw_ptr(&self) -> *mut libjulius_sys::Recog {
        self.0 as *const libjulius_sys::Recog as *mut libjulius_sys::Recog
    }
}

impl<'a> Drop for Recog<'a> {
    fn drop(&mut self) {
        unsafe {
            libjulius_sys::j_recog_free(&mut *self.0);
        }
    }
}
