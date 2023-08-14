use strum_macros::FromRepr;

use crate::{iter::BindIterator, sentence_align::SentenceAlign};

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ResultStatus {
    RejectLong = libjulius_sys::J_RESULT_STATUS_REJECT_LONG,
    BufferOverflow = libjulius_sys::J_RESULT_STATUS_BUFFER_OVERFLOW,
    RejectPower = libjulius_sys::J_RESULT_STATUS_REJECT_POWER,
    Terminate = libjulius_sys::J_RESULT_STATUS_TERMINATE,
    OnlySilence = libjulius_sys::J_RESULT_STATUS_ONLY_SILENCE,
    RejectGmm = libjulius_sys::J_RESULT_STATUS_REJECT_GMM,
    RejectShort = libjulius_sys::J_RESULT_STATUS_REJECT_SHORT,
    Fail = libjulius_sys::J_RESULT_STATUS_FAIL,
    Success = libjulius_sys::J_RESULT_STATUS_SUCCESS as i32,
}

#[repr(i16)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum AlignUnitType {
    Word = libjulius_sys::PER_WORD as i16,
    Phoneme = libjulius_sys::PER_PHONEME as i16,
    State = libjulius_sys::PER_STATE as i16,
}

#[derive(Debug)]
pub struct RecogProcess(libjulius_sys::RecogProcess);

impl RecogProcess {
    pub(crate) fn new(process: libjulius_sys::RecogProcess) -> Self {
        Self(process)
    }
    pub fn is_live(&self) -> bool {
        self.0.live == 1
    }
    pub fn result(&self) -> Output {
        let result = self.0.result;
        Output(result)
    }
}

#[derive(Debug)]
pub struct Output(libjulius_sys::Output);

impl Output {
    pub fn status(&self) -> ResultStatus {
        ResultStatus::from_repr(self.0.status).unwrap()
    }
    pub fn get_sent(&self) -> &[Sentence] {
        unsafe {
            let s = std::slice::from_raw_parts(self.0.sent, self.0.sentnum as usize);
            let (head, body, _tail) = s.align_to::<Sentence>();
            assert!(head.is_empty(), "Sentence is not aligned");
            body
        }
    }
}

#[derive(Debug)]
pub struct Sentence(libjulius_sys::Sentence);

impl Sentence {
    pub fn get_align(&self) -> BindIterator<libjulius_sys::SentenceAlign, SentenceAlign> {
        BindIterator::new(
            self.0.align,
            Box::new(|curr| unsafe { (*curr).next }),
            Box::new(|curr| SentenceAlign(unsafe { *curr })),
        )
    }
}
