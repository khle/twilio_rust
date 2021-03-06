extern crate hyper;

use std::str;
use ::{Client, ToUrlEncoded};
use serde_helper;
use chrono::prelude::*;
use futures::{Future, future};
use hyper::{Body, Method, Request};
use hyper::header::{ContentType, ContentLength};
use url::{form_urlencoded, Url};

pub struct Calls<'a> {
    client: &'a Client,
}

#[derive(Deserialize)]
pub struct Call {
    pub sid: String,
    pub account_sid: String,
    pub parent_call_sid: Option<String>,
    pub from: String,
    pub to: String,
    pub phone_number_sid: Option<String>,
    pub status: CallStatus,
    pub duration: Option<String>,
    pub answered_by: Option<String>,
    pub price: Option<String>,
    pub price_unit: Option<String>,
    pub direction: Option<Direction>,
    pub forwarded_from: Option<String>,
    pub to_formatted: Option<String>,
    pub from_formatted: Option<String>,
    pub caller_name: Option<String>,
    #[serde(deserialize_with = "serde_helper::deserialize_rfc2822")] pub date_created: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "serde_helper::deserialize_rfc2822")] pub date_updated: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "serde_helper::deserialize_rfc2822")] pub start_time: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "serde_helper::deserialize_rfc2822")] pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Queued,
    Ringing,
    #[serde(rename = "in-progress")] InProgress,
    Canceled,
    Completed,
    Busy,
    Failed,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    Inbound,
    OutboundApi,
    OutboundDial,
    TrunkingTerminating,
    TrunkingOriginating,
}

#[derive(Copy, Clone)]
pub enum TwimlSource<'a> {
    Url(&'a Url),
    ApplicationSid(&'a str),
}

#[derive(Copy, Clone)]
pub enum CallbackMethod {
    Post,
    Get,
}

impl CallbackMethod {
    pub fn name(&self) -> &str {
        match *self {
            CallbackMethod::Post => "POST",
            CallbackMethod::Get => "GET",
        }
    }
}

pub enum StatusCallbackEvent {
    Initiated,
    Ringing,
    Answered,
    Completed,
}

impl StatusCallbackEvent {
    pub fn name(&self) -> &str {
        match *self {
            StatusCallbackEvent::Initiated => "initiated",
            StatusCallbackEvent::Ringing => "ringing",
            StatusCallbackEvent::Answered => "answered",
            StatusCallbackEvent::Completed => "completed",
        }
    }
}

#[derive(Copy, Clone)]
pub enum RecordingChannel {
    Mono,
    Dual,
}

impl RecordingChannel {
    pub fn name(&self) -> &str {
        match *self {
            RecordingChannel::Mono => "mono",
            RecordingChannel::Dual => "dual",
        }
    }
}

#[derive(Copy, Clone)]
pub enum ModifyCallStatus {
    Canceled,
    Completed,
}

impl ModifyCallStatus {
    pub fn name(&self) -> &str {
        match *self {
            ModifyCallStatus::Canceled => "canceled",
            ModifyCallStatus::Completed => "completed",
        }
    }
}

pub struct OutboundCall<'a> {
    from: &'a str,
    to: &'a str,
    twiml_source: TwimlSource<'a>,
    method: Option<CallbackMethod>,
    fallback_url: Option<&'a Url>,
    fallback_method: Option<CallbackMethod>,
    status_callback: Option<&'a Url>,
    status_callback_method: Option<CallbackMethod>,
    status_callback_event: &'a [StatusCallbackEvent],
    send_digits: Option<&'a str>,
    timeout: Option<u32>,
    record: Option<bool>,
    recording_channels: Option<RecordingChannel>,
    recording_status_callback: Option<&'a Url>,
    recording_status_callback_method: Option<CallbackMethod>,
}

pub struct OutboundCallBuilder<'a> {
    from: &'a str,
    to: &'a str,
    twiml_source: TwimlSource<'a>,
    method: Option<CallbackMethod>,
    fallback_url: Option<&'a Url>,
    fallback_method: Option<CallbackMethod>,
    status_callback: Option<&'a Url>,
    status_callback_method: Option<CallbackMethod>,
    status_callback_event: &'a [StatusCallbackEvent],
    send_digits: Option<&'a str>,
    timeout: Option<u32>,
    record: Option<bool>,
    recording_channels: Option<RecordingChannel>,
    recording_status_callback: Option<&'a Url>,
    recording_status_callback_method: Option<CallbackMethod>,
}

impl<'a> OutboundCallBuilder<'a> {
    pub fn new(from: &'a str, to: &'a str, url: &'a Url) -> OutboundCallBuilder<'a> {
       OutboundCallBuilder {
           from,
           to,
           twiml_source: TwimlSource::Url(url),
           method: None,
           fallback_url: None,
           fallback_method: None,
           status_callback: None,
           status_callback_method: None,
           status_callback_event: &[],
           send_digits: None,
           timeout: None,
           record: None,
           recording_channels: None,
           recording_status_callback: None,
           recording_status_callback_method: None,
       }
    }

    pub fn with_method(&mut self, method: CallbackMethod) -> &mut Self {
        self.method = Some(method);
        self
    }

    pub fn with_fallback_url(&mut self, fallback_url: &'a Url) -> &mut Self {
        self.fallback_url = Some(fallback_url);
        self
    }

    pub fn with_fallback_method(&mut self, fallback_method: CallbackMethod) -> &mut Self {
        self.fallback_method = Some(fallback_method);
        self
    }

    pub fn with_status_callback(&mut self, status_callback: &'a Url) -> &mut Self {
        self.status_callback = Some(status_callback);
        self
    }

    pub fn with_status_callback_events(&mut self, events: &'a [StatusCallbackEvent]) -> &mut Self {
        self.status_callback_event = events;
        self
    }

    pub fn with_send_digits(&mut self, digits: &'a str) -> &mut Self {
        self.send_digits = Some(digits);
        self
    }

    pub fn with_timeout(&mut self, timeout: u32) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_record(&mut self, record: bool) -> &mut Self {
        self.record = Some(record);
        self
    }

    pub fn with_recording_channels(&mut self, recording_channels: RecordingChannel) -> &mut Self {
        self.recording_channels = Some(recording_channels);
        self
    }

    pub fn with_recording_status_callback(&mut self, url: &'a Url) -> &mut Self {
        self.recording_status_callback = Some(url);
        self
    }

    pub fn with_recording_status_callback_method(&mut self, method: CallbackMethod) -> &mut Self {
        self.recording_status_callback_method = Some(method);
        self
    }

    pub fn build(&mut self) -> OutboundCall<'a> {
        OutboundCall {
            from: self.from,
            to: self.to,
            twiml_source: self.twiml_source,
            method: self.method,
            fallback_url: self.fallback_url,
            fallback_method: self.fallback_method,
            status_callback: self.status_callback,
            status_callback_method: self.status_callback_method,
            status_callback_event: self.status_callback_event,
            send_digits: self.send_digits,
            timeout: self.timeout,
            record: self.record,
            recording_channels: self.recording_channels,
            recording_status_callback: self.recording_status_callback,
            recording_status_callback_method: self.recording_status_callback_method,
        }
    }
}

impl<'a> ToUrlEncoded for OutboundCall<'a> {

    fn to_url_encoded(&self) -> String {
        let mut encoder = form_urlencoded::Serializer::new(String::new());
        encoder.append_pair("From", self.from);
        encoder.append_pair("To", self.to);
        let (name, value) = match self.twiml_source {
            TwimlSource::Url(x) => ("Url", x.as_str()),
            TwimlSource::ApplicationSid(x) => ("ApplicationSid", x),
        };
        encoder.append_pair(name, value);
        if let Some(ref x) = self.method {
            encoder.append_pair("Method", x.name());
        }

        if let Some(url) = self.fallback_url {
            encoder.append_pair("FallbackUrl", url.as_str());
        }
        if let Some(ref x) = self.fallback_method {
            encoder.append_pair("FallbackMethod", x.name());
        }

        if let Some(url) = self.status_callback {
            encoder.append_pair("StatusCallback", url.as_str());
        }
        if let Some(ref x) = self.status_callback_method {
            encoder.append_pair("StatusCallbackMethod", x.name());
        }
        for e in self.status_callback_event.iter() {
            encoder.append_pair("StatusCallbackEvent", e.name());
        }

        if let Some(digits) = self.send_digits {
            encoder.append_pair("SendDigits", digits);
        }
        if let Some(timeout) = self.timeout {
            encoder.append_pair("Timeout", &timeout.to_string());
        }
        if let Some(record) = self.record {
            encoder.append_pair("Record", &record.to_string());
        }
        if let Some(recording_channel) = self.recording_channels {
            encoder.append_pair("RecordingChannels", recording_channel.name());
        }
        if let Some(recording_status_callback) = self.recording_status_callback {
            encoder.append_pair("RecordingStatusCallback", recording_status_callback.as_str());
        }
        if let Some(recording_status_callback_method) = self.recording_status_callback_method {
            encoder.append_pair("RecordingStatusCallbackMethod", recording_status_callback_method.name());
        }
        encoder.finish()
    }
}

impl<'a> Calls<'a> {

    pub fn new(client: &Client) -> Calls {
        Calls { client }
    }

    pub fn get_call(
        &self,
        call_sid: &str,
    ) -> Box<Future<Item = Call, Error = ::TwilioError>> {
        let uri = format!(
            "{}/2010-04-01/Accounts/{}/Calls/{}.json",
            ::BASE_URI,
            self.client.account_sid,
            call_sid
        ).parse()
            .unwrap();
        let req: Request<Body> = Request::new(Method::Get, uri);
        self.client.make_req(req)
    }

    pub fn make_call(&self, outbound_call: &OutboundCall) -> Box<Future<Item = Call, Error = ::TwilioError>> {
        let encoded_params = outbound_call.to_url_encoded();
        let uri = format!(
            "{}/2010-04-01/Accounts/{}/Calls.json",
            ::BASE_URI,
            self.client.account_sid).parse().unwrap();
        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::form_url_encoded());
        req.headers_mut().set(ContentLength(encoded_params.len() as u64));
        req.set_body(encoded_params.into_bytes());
        self.client.make_req(req)
    }

    pub fn get_calls(&self) -> Box<Future<Item = ::Page<Call>, Error = ::TwilioError>> {
        let uri = format!(
            "{}/2010-04-01/Accounts/{}/Calls.json",
            ::BASE_URI,
            self.client.account_sid).parse().unwrap();
        let req = Request::new(Method::Get, uri);
        self.client.get_page(req)
    }

    pub fn get_calls_with_page_size(&self, page_size: u16) -> Box<Future<Item = ::Page<Call>, Error = ::TwilioError>> {
        let uri = format!(
            "{}/2010-04-01/Accounts/{}/Calls.json?PageSize={}",
            ::BASE_URI,
            self.client.account_sid, page_size).parse().unwrap();
        let req = Request::new(Method::Get, uri);
        self.client.get_page(req)
    }

    pub fn get_next_page(&self, page: &::Page<Call>) -> Box<Future<Item = Option<::Page<Call>>, Error = ::TwilioError>> {
        match page.next_page_uri.as_ref() {
            Some(uri) => {
                let req = Request::new(Method::Get, uri.clone());
                Box::new(self.client.get_page(req).map(|p| Some(p)))
            },
            None => Box::new(future::ok(None))
        }
    }

    pub fn redirect_call(&self, call_sid: &str, redirect_url: &Url,
                         redirect_method: Option<CallbackMethod>) -> Box<Future<Item = Call, Error = ::TwilioError>> {

        let mut encoder = form_urlencoded::Serializer::new(String::new());
        encoder.append_pair("Url", redirect_url.as_str());
        if let Some(method) = redirect_method {
            encoder.append_pair("Method", method.name());
        }
        let params: String = encoder.finish();
        let uri = format!(
            "{}/2010-04-01/Accounts/{}/Calls/{}.json",
            ::BASE_URI,
            self.client.account_sid,
            call_sid
        ).parse()
            .unwrap();
        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::form_url_encoded());
        req.headers_mut().set(ContentLength(params.len() as u64));
        req.set_body(params.into_bytes());
        self.client.make_req(req)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json;

    #[test]
    fn test_callstatus_deserialize() {
        assert_eq!(CallStatus::Queued, serde_json::from_str("\"queued\"").unwrap());
        assert_eq!(CallStatus::InProgress, serde_json::from_str("\"in-progress\"").unwrap());
    }

    #[test]
    fn test_direction_deserialize() {
        assert_eq!(Direction::TrunkingTerminating, serde_json::from_str("\"trunking-terminating\"").unwrap());
    }

    #[test]
    fn test_url_encoding() {
        let url = Url::parse("http://www.example.com").unwrap();
        let outbound_call = OutboundCallBuilder::new("tom", "jerry", &url).build();
        let url_encoded = outbound_call.to_url_encoded();
        assert_eq!("From=tom&To=jerry&Url=http%3A%2F%2Fwww.example.com%2F", &url_encoded);
    }

    #[test]
    fn test_status_callback() {
        let url = Url::parse("http://www.example.com").unwrap();
        let events = [StatusCallbackEvent::Answered, StatusCallbackEvent::Ringing];
        let outbound_call = OutboundCallBuilder::new("tom", "jerry", &url)
            .with_status_callback_events(&events)
            .build();
        let url_encoded = outbound_call.to_url_encoded();
        assert_eq!("From=tom&To=jerry&Url=http%3A%2F%2Fwww.example.com%2F\
            &StatusCallbackEvent=answered&StatusCallbackEvent=ringing", &url_encoded);
    }


}