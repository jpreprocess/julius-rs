use std::{ffi::CString, fmt::Debug, str::Utf8Error};

#[derive(Debug)]
pub struct SentenceAlign(pub(crate) libjulius_sys::SentenceAlign);

impl SentenceAlign {
    pub fn t(&self) -> SentenceAlignWithType<'_> {
        match self.0.unittype as u32 {
            libjulius_sys::PER_WORD => SentenceAlignWithType::Word(WordFrame(self)),
            libjulius_sys::PER_PHONEME => SentenceAlignWithType::Phoneme(PhonemeFrame(self)),
            libjulius_sys::PER_STATE => SentenceAlignWithType::State(StateFrame(self)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum SentenceAlignWithType<'a> {
    Word(WordFrame<'a>),
    Phoneme(PhonemeFrame<'a>),
    State(StateFrame<'a>),
}

pub trait IAlignFrame {
    fn sentence_align(&self) -> &SentenceAlign;
    fn allscore(&self) -> f32 {
        self.sentence_align().0.allscore
    }
}

#[derive(Debug)]
pub struct WordFrame<'a>(&'a SentenceAlign);
impl<'a> IAlignFrame for WordFrame<'a> {
    fn sentence_align(&self) -> &SentenceAlign {
        self.0
    }
}
impl<'a> WordFrame<'a> {
    pub fn frame_iter(&self) -> impl Iterator<Item = SentenceAlignFrameWord> + '_ {
        (0..self.0 .0.num as usize).map(|i| unsafe {
            SentenceAlignFrameWord {
                w: *self.0 .0.w.add(i),
                begin_frame: *self.0 .0.begin_frame.add(i),
                end_frame: *self.0 .0.end_frame.add(i),
                avgscore: *self.0 .0.avgscore.add(i),
            }
        })
    }
}

#[derive(Debug)]
pub struct PhonemeFrame<'a>(&'a SentenceAlign);
impl<'a> IAlignFrame for PhonemeFrame<'a> {
    fn sentence_align(&self) -> &SentenceAlign {
        self.0
    }
}
impl<'a> PhonemeFrame<'a> {
    pub fn frame_iter(&self) -> impl Iterator<Item = SentenceAlignFramePhoneme> + '_ {
        (0..self.0 .0.num as usize).map(|i| unsafe {
            SentenceAlignFramePhoneme {
                ph: HMMLogical(**self.0 .0.ph.add(i)),
                begin_frame: *self.0 .0.begin_frame.add(i),
                end_frame: *self.0 .0.end_frame.add(i),
                avgscore: *self.0 .0.avgscore.add(i),
            }
        })
    }
}

#[derive(Debug)]
pub struct StateFrame<'a>(&'a SentenceAlign);
impl<'a> IAlignFrame for StateFrame<'a> {
    fn sentence_align(&self) -> &SentenceAlign {
        self.0
    }
}
impl<'a> StateFrame<'a> {
    pub fn frame_iter(&self) -> impl Iterator<Item = SentenceAlignFrameState> + '_ {
        let is_multipath = !self.0 .0.is_iwsp.is_null();
        (0..self.0 .0.num as usize)
            .map(move |i| unsafe {
                SentenceAlignFrameState {
                    ph: HMMLogical(**self.0 .0.ph.add(i)),
                    loc: *self.0 .0.loc.add(i),
                    is_iwsp: if is_multipath {
                        Some(*self.0 .0.is_iwsp.add(i) != 0)
                    } else {
                        None
                    },
                    begin_frame: *self.0 .0.begin_frame.add(i),
                    end_frame: *self.0 .0.end_frame.add(i),
                    avgscore: *self.0 .0.avgscore.add(i),
                }
            })
    }
}

#[derive(Debug)]
pub struct SentenceAlignFrameWord {
    pub begin_frame: i32,
    pub end_frame: i32,
    pub avgscore: f32,
    pub w: i32,
}

#[derive(Debug)]
pub struct SentenceAlignFramePhoneme {
    pub begin_frame: i32,
    pub end_frame: i32,
    pub avgscore: f32,
    pub ph: HMMLogical,
}

#[derive(Debug)]
pub struct SentenceAlignFrameState {
    pub begin_frame: i32,
    pub end_frame: i32,
    pub avgscore: f32,
    pub ph: HMMLogical,
    pub loc: i16,
    pub is_iwsp: Option<bool>,
}

pub struct HMMLogical(libjulius_sys::HMM_Logical);
impl HMMLogical {
    pub fn name(&self) -> Result<String, Utf8Error> {
        let cstring = unsafe { CString::from_raw(self.0.name) };
        let string = cstring.to_str()?.to_string();
        std::mem::forget(cstring);
        Ok(string)
    }
    pub fn is_pseudo(&self) -> bool {
        self.0.is_pseudo != 0
    }
    pub fn defined(&self) -> &HTKHMMData {
        unsafe { &*(self.0.body.defined as *const HTKHMMData) }
    }
    pub fn cd_set(&self) -> &CDSet {
        unsafe { &*(self.0.body.defined as *const CDSet) }
    }
}
impl Debug for HMMLogical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { CString::from_raw(self.0.name) };
        let is_pseudo = self.is_pseudo();
        let result = if is_pseudo {
            f.debug_struct("HMMLogical")
                .field("name", &name)
                .field("is_pseudo", &is_pseudo)
                .field("body", self.defined())
                .finish()
        } else {
            f.debug_struct("HMMLogical")
                .field("name", &name)
                .field("is_pseudo", &is_pseudo)
                .field("body", self.cd_set())
                .finish()
        };
        std::mem::forget(name);
        result
    }
}

#[derive(Debug)]
pub struct HTKHMMData(libjulius_sys::HTK_HMM_Data);
impl HTKHMMData {
    pub fn name(&self) -> Result<String, Utf8Error> {
        let cstring = unsafe { CString::from_raw(self.0.name) };
        let string = cstring.to_str()?.to_string();
        std::mem::forget(cstring);
        Ok(string)
    }
}

#[derive(Debug)]
pub struct CDSet(libjulius_sys::CD_Set);
impl CDSet {
    pub fn name(&self) -> Result<String, Utf8Error> {
        let cstring = unsafe { CString::from_raw(self.0.name) };
        let string = cstring.to_str()?.to_string();
        std::mem::forget(cstring);
        Ok(string)
    }
}
