use crate::ease;

use std::time::{Duration, Instant};

pub enum AnimationReq {
    Delete {
        desired_len: usize,
        animation_duration: Duration,
    },
    Wait {
        wait_time: Duration,
    },
    Append {
        additional_chars: String,
        animation_duration: Duration,
    },
}

pub enum Animation {
    Delete(DeleteOverTime),
    Append(AppendOverTime),
    Wait(String, Instant),
    None(String),
}

impl Animation {
    pub fn finished(&self, now: Instant) -> bool {
        match self {
            Animation::Delete(d) => d.finished(now),
            Animation::Append(d) => d.finished(now),
            Animation::Wait(_, t) => now > *t,
            Animation::None(_) => true,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Animation::Wait(s, _) => s,
            Animation::None(s) => s,
            Animation::Delete(d) => d.as_str(),
            Animation::Append(a) => a.as_str(),
        }
    }

    pub fn into_finished_string(self) -> String {
        match self {
            Animation::Wait(s, _) => s,
            Animation::None(s) => s,
            Animation::Delete(d) => d.into_finished_string(),
            Animation::Append(a) => a.into_finished_string(),
        }
    }

    pub fn update(&mut self, now: Instant) {
        match self {
            Animation::Delete(d) => d.update(now),
            Animation::Append(a) => a.update(now),
            _ => (),
        }
    }
}

pub struct DeleteOverTime {
    s: String,
    start_len: usize,
    desired_len: usize,
    animation_start: Instant,
    animation_duration: Duration,
}

impl DeleteOverTime {
    pub fn update(&mut self, now: Instant) {
        let time_factor = self.time_factor(now);

        let delete_factor = ease::in_sine(time_factor);
        let deleted_chars = ((self.start_len - self.desired_len) as f32 * delete_factor) as usize;
        let desired_current_len = self.start_len - deleted_chars;

        self.s.truncate(desired_current_len);
    }

    pub fn time_factor(&self, now: Instant) -> f32 {
        let duration_since_start = now - self.animation_start;
        (duration_since_start.as_secs_f32() / self.animation_duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    pub fn finished(&self, now: Instant) -> bool {
        self.time_factor(now) >= 1.0f32
    }

    pub fn as_str(&self) -> &str {
        &self.s
    }

    pub fn into_finished_string(mut self) -> String {
        self.update(self.animation_start + self.animation_duration);
        self.s
    }
}

use std::collections::VecDeque;

pub struct AppendOverTime {
    s: String,
    start_len: usize,
    additional_characters: VecDeque<char>,
    animation_start: Instant,
    animation_duration: Duration,
}

impl AppendOverTime {
    pub fn update(&mut self, now: Instant) {
        let time_factor = self.time_factor(now);

        let append_factor = ease::in_sine(time_factor);
        let current_len = self.s.chars().count();
        let final_len = current_len + self.additional_characters.len();
        let desired_len =
            ((final_len - self.start_len) as f32 * append_factor) as usize + self.start_len;

        while self.s.len() < desired_len {
            self.s.push(
                self.additional_characters
                    .pop_front()
                    .expect("Attempted to pop too many"),
            );
        }
    }

    pub fn time_factor(&self, now: Instant) -> f32 {
        let duration_since_start = now - self.animation_start;
        (duration_since_start.as_secs_f32() / self.animation_duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    fn finished(&self, now: Instant) -> bool {
        self.time_factor(now) >= 1.0f32
    }

    pub fn as_str(&self) -> &str {
        &self.s
    }

    pub fn into_finished_string(mut self) -> String {
        self.update(self.animation_start + self.animation_duration);
        self.s
    }
}

pub fn apply_animation_req(req: AnimationReq, s: String, now: Instant) -> Animation {
    let s_len = s.len();
    match req {
        AnimationReq::Delete {
            desired_len,
            animation_duration,
        } => Animation::Delete(DeleteOverTime {
            s,
            start_len: s_len,
            desired_len,
            animation_start: now,
            animation_duration,
        }),
        AnimationReq::Append {
            additional_chars,
            animation_duration,
        } => Animation::Append(AppendOverTime {
            s,
            start_len: s_len,
            additional_characters: additional_chars.chars().collect(),
            animation_start: now,
            animation_duration,
        }),
        AnimationReq::Wait { wait_time } => Animation::Wait(s, now + wait_time),
    }
}

pub fn construct_animation_requests(current: &str, desired: &str) -> VecDeque<AnimationReq> {
    let mut ret = VecDeque::new();
    let first_differing_char = current
        .chars()
        .zip(desired.chars())
        .position(|(a, b)| a != b);

    let first_differing_char = first_differing_char.unwrap_or(0);

    if !current.is_empty() {
        ret.push_back(AnimationReq::Wait {
            wait_time: Duration::from_secs_f32(1.5),
        });
        ret.push_back(AnimationReq::Delete {
            desired_len: first_differing_char,
            animation_duration: Duration::from_secs_f32(1.5),
        });
    }

    ret.push_back(AnimationReq::Append {
        additional_chars: desired.chars().skip(first_differing_char).collect(),
        animation_duration: Duration::from_secs_f32(1.5),
    });

    ret
}
