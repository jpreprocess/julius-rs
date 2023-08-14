use std::error::Error;

use julius::{
    sentence_align::{HMMLogical, SentenceAlignWithType},
    JConf, Recog,
};

fn main() -> Result<(), Box<dyn Error>> {
    let config = "
-h models/hmmdefs_monof_mix16_gid.binhmm
-dfa wav/sample.dfa
-v wav/sample.dict
-palign
-input file
";
    let wav_path = "wav/sample.wav";

    let jconf = JConf::from_string(&config)?;
    let mut recog = Recog::from_jconf(jconf)?;
    recog.add_callback(julius::CallbackType::Result, cb);
    recog.adin_init()?;
    recog.open_stream(Some(&wav_path))?;
    recog.recognize_stream()?;
    recog.close_stream()?;

    Ok(())
}

fn cb(recog: &mut Recog) {
    for r in recog.get_processes() {
        if !r.is_live() {
            continue;
        }

        let result = r.result();
        match result.status() {
            julius::recog_process::ResultStatus::RejectPower => {
                println!("<input rejected by power>")
            }
            julius::recog_process::ResultStatus::Terminate => {
                println!("<input teminated by request>")
            }
            julius::recog_process::ResultStatus::OnlySilence => {
                println!("<input rejected by decoder (silence input result)>")
            }
            julius::recog_process::ResultStatus::RejectGmm => {
                println!("<input rejected by GMM>")
            }
            julius::recog_process::ResultStatus::RejectShort => {
                println!("<input rejected by short input>")
            }
            julius::recog_process::ResultStatus::RejectLong => {
                println!("<input rejected by long input>")
            }
            julius::recog_process::ResultStatus::Fail => println!("<search failed>"),

            _ => (),
        }
        for s in result.get_sent() {
            for a in s.get_align() {
                match a.t() {
                    SentenceAlignWithType::Word(word_frame) => {
                        for word in word_frame.frame_iter() {
                            println!(
                                "[{} {}]{} {}",
                                word.begin_frame, word.end_frame, word.avgscore, word.w
                            )
                        }
                    }
                    SentenceAlignWithType::Phoneme(phoneme_frame) => {
                        for phoneme in phoneme_frame.frame_iter() {
                            let ph = &phoneme.ph;
                            let ph_str = display_hmmlogical(ph);
                            println!(
                                "[{} {}]{} {}",
                                phoneme.begin_frame, phoneme.end_frame, phoneme.avgscore, ph_str
                            )
                        }
                    }
                    SentenceAlignWithType::State(state_frame) => {
                        for state in state_frame.frame_iter() {
                            let ph = &state.ph;
                            let ph_str = display_hmmlogical(ph);
                            println!(
                                "[{} {}]{} {} #{}",
                                state.begin_frame,
                                state.end_frame,
                                state.avgscore,
                                ph_str,
                                state.loc
                            )
                        }
                    }
                }
            }
        }
    }
}

fn display_hmmlogical(ph: &HMMLogical) -> String {
    if ph.is_pseudo() {
        format!("{{{}}}", ph.name().unwrap())
    } else if ph.name() == ph.defined().name() {
        format!("{}", ph.name().unwrap(),)
    } else {
        format!("{}[{}]", ph.name().unwrap(), ph.defined().name().unwrap())
    }
}
