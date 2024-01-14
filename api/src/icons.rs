use tera::{Value, to_value, Error};
use std::collections::HashMap;

pub fn sbubby(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value("sbubby".to_string())?)
}

pub fn icon_home(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
        <svg viewBox="0 0 64 64" class="svg-icon icon-home">
            <path d="M61.2,21.2L35.4,4.6c-2.1-1.3-4.8-1.3-6.8,0L2.8,21.2c-1,0.7-1.3,2.1-0.7,3.1c0.7,1,2.1,1.3,3.1,0.7l1.7-1.1v30.1
                c0,3.5,2.8,6.3,6.3,6.3h37.6c3.5,0,6.3-2.8,6.3-6.3V23.9l1.7,1.1c0.4,0.2,0.8,0.4,1.2,0.4c0.7,0,1.5-0.4,1.9-1
                C62.6,23.3,62.3,21.9,61.2,21.2z M52.6,54.1c0,1-0.8,1.8-1.8,1.8H13.2c-1,0-1.8-0.8-1.8-1.8V21L31,8.4c0.6-0.4,1.4-0.4,2,0L52.6,21
                V54.1z"/>
            <path class="opt" d="M27.2,24.6c-2.2,0-4.3,0.9-5.8,2.4c-3.2,3.2-3.2,8.4,0,11.6l0.6,0.6c0,0,0,0,0,0l8.4,8.5c0.4,0.4,1,0.7,1.6,0.7
                s1.2-0.2,1.6-0.7l8.4-8.5c0,0,0,0,0,0l0.6-0.6c1.5-1.6,2.4-3.6,2.4-5.8c0-2.2-0.8-4.2-2.4-5.8c-1.5-1.6-3.6-2.4-5.8-2.4
                c0,0,0,0,0,0c-1.7,0-3.4,0.5-4.8,1.6C30.6,25.2,29,24.6,27.2,24.6z M34.2,30.2c0.7-0.7,1.6-1.1,2.6-1.1c0,0,0,0,0,0
                c1,0,1.9,0.4,2.6,1.1c0.7,0.7,1.1,1.6,1.1,2.6c0,1-0.4,1.9-1.1,2.7L32,43l-6.8-6.8l-0.6-0.6c-1.4-1.4-1.4-3.8,0-5.3
                c1.4-1.4,3.8-1.4,5.2,0l0.6,0.6c0.4,0.4,1,0.7,1.6,0.7h0c0.6,0,1.2-0.2,1.6-0.7L34.2,30.2z"/>
        </svg>"#.to_string())?)
}

pub fn icon_profile(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-profile">
        <path class="opt" d="M45.1016 1.80078H18.9016C15.5016 1.80078 12.6016 4.60078 12.6016 8.10078V56.1008C12.6016 59.5008 15.4016 62.4008 18.9016 62.4008H45.1016C48.5016 62.4008 51.4016 59.6008 51.4016 56.1008V8.00078C51.3016 4.60078 48.5016 1.80078 45.1016 1.80078ZM46.8016 56.0008C46.8016 57.0008 46.0016 57.8008 45.0016 57.8008H18.9016C17.9016 57.8008 17.1016 57.0008 17.1016 56.0008V8.00078C17.1016 7.00078 17.9016 6.20078 18.9016 6.20078H45.1016C46.1016 6.20078 46.9016 7.00078 46.9016 8.00078V56.0008H46.8016Z"/>
        <path d="M32 43.3008C35.9 43.3008 39 40.2008 39 36.3008C39 32.4008 35.9 29.3008 32 29.3008C28.1 29.3008 25 32.4008 25 36.3008C25 40.1008 28.1 43.3008 32 43.3008ZM32 33.7008C33.4 33.7008 34.5 34.8008 34.5 36.2008C34.5 37.6008 33.4 38.7008 32 38.7008C30.6 38.7008 29.5 37.6008 29.5 36.2008C29.5 34.8008 30.6 33.7008 32 33.7008Z"/>
        <path d="M32.1031 44.4004C27.8031 44.4004 23.9031 46.6004 21.6031 50.2004C20.9031 51.2004 21.2031 52.6004 22.3031 53.3004C22.7031 53.5004 23.1031 53.7004 23.5031 53.7004C24.2031 53.7004 25.0031 53.3004 25.4031 52.7004C26.9031 50.4004 29.4031 49.0004 32.1031 49.0004C34.6031 49.0004 37.1031 50.4004 38.7031 52.8004C39.4031 53.8004 40.8031 54.1004 41.8031 53.4004C42.8031 52.7004 43.1031 51.3004 42.4031 50.3004C39.9031 46.6004 36.1031 44.4004 32.1031 44.4004Z"/>
        <path d="M27.6008 18.6012H29.8008V20.8012C29.8008 22.0012 30.8008 23.1012 32.1008 23.1012C33.3008 23.1012 34.4008 22.1012 34.4008 20.8012V18.6012H36.6008C37.8008 18.6012 38.9008 17.6012 38.9008 16.3012C38.9008 15.0012 37.9008 14.0012 36.6008 14.0012H34.4008V12.0012C34.4008 10.8012 33.4008 9.70117 32.1008 9.70117C30.9008 9.70117 29.8008 10.7012 29.8008 12.0012V14.2012H27.6008C26.4008 14.2012 25.3008 15.2012 25.3008 16.5012C25.3008 17.8012 26.3008 18.6012 27.6008 18.6012Z"/>
    </svg>"#.to_string())?)
}

pub fn icon_applications(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-applications">
        <path d="M23.9,1.8H8C4.6,1.8,1.8,4.6,1.8,8v15.9c0,3.4,2.8,6.3,6.3,6.3h15.9c3.4,0,6.3-2.8,6.3-6.3V8C30.1,4.6,27.3,1.8,23.9,1.8z
            M25.6,23.9c0,1-0.8,1.8-1.8,1.8H8c-1,0-1.8-0.8-1.8-1.8V8C6.3,7,7,6.3,8,6.3h15.9c1,0,1.8,0.8,1.8,1.8V23.9z"/>
        <path d="M56,1.8H40.1c-3.4,0-6.3,2.8-6.3,6.3v15.9c0,3.4,2.8,6.3,6.3,6.3H56c3.4,0,6.3-2.8,6.3-6.3V8C62.3,4.6,59.4,1.8,56,1.8z
            M57.8,23.9c0,1-0.8,1.8-1.8,1.8H40.1c-1,0-1.8-0.8-1.8-1.8V8c0-1,0.8-1.8,1.8-1.8H56c1,0,1.8,0.8,1.8,1.8V23.9z"/>
        <path class="opt" d="M23.9,33.9H8c-3.4,0-6.3,2.8-6.3,6.3V56c0,3.4,2.8,6.3,6.3,6.3h15.9c3.4,0,6.3-2.8,6.3-6.3V40.1
            C30.1,36.7,27.3,33.9,23.9,33.9z M25.6,56c0,1-0.8,1.8-1.8,1.8H8c-1,0-1.8-0.8-1.8-1.8V40.1c0-1,0.8-1.8,1.8-1.8h15.9
            c1,0,1.8,0.8,1.8,1.8V56z"/>
    </svg>"#.to_string())?)
}

pub fn icon_relationships(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-relationships">
        <path d="M21.8,36.8c6.9,0,12.4-5.6,12.4-12.4s-5.6-12.4-12.4-12.4S9.4,17.5,9.4,24.4S15,36.8,21.8,36.8z M21.8,16.4
            c4.4,0,7.9,3.6,7.9,7.9s-3.6,7.9-7.9,7.9c-4.4,0-7.9-3.6-7.9-7.9S17.4,16.4,21.8,16.4z"/>
        <path d="M21.8,39.9c-7.2,0-14.1,2.9-19.4,8.3c-0.9,0.9-0.9,2.3,0,3.2c0.4,0.4,1,0.7,1.6,0.7c0.6,0,1.2-0.2,1.6-0.7
            c4.4-4.5,10.2-7,16.2-7c5.9,0,11.7,2.5,16.2,7c0.9,0.9,2.3,0.9,3.2,0c0.9-0.9,0.9-2.3,0-3.2C35.9,42.9,29,39.9,21.8,39.9z"/>
        <path class="opt" d="M47.3,36.8c4,0,7.3-3.3,7.3-7.3c0-4-3.3-7.3-7.3-7.3s-7.3,3.3-7.3,7.3C39.9,33.5,43.2,36.8,47.3,36.8z M47.3,26.6
            c1.6,0,2.8,1.3,2.8,2.8c0,1.6-1.3,2.8-2.8,2.8s-2.8-1.3-2.8-2.8C44.4,27.9,45.7,26.6,47.3,26.6z"/>
        <path class="opt" d="M61.5,45.6c-5.3-4.9-12.6-6.9-19.9-5c-1.2,0.3-1.9,1.5-1.6,2.7c0.3,1.2,1.6,1.9,2.7,1.6c5.8-1.5,11.6,0,15.7,3.9
            c0.4,0.4,1,0.6,1.5,0.6c0.6,0,1.2-0.2,1.6-0.7C62.5,47.9,62.4,46.5,61.5,45.6z"/>
    </svg>"#.to_string())?)
}

pub fn icon_search(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-search">
        <path class="opt" d="M49.3008 20.6992C48.2008 20.6992 47.1008 20.7992 46.1008 21.0992C45.2008 13.2992 38.6008 7.19922 30.6008 7.19922C24.9008 7.19922 19.7008 10.2992 17.0008 15.2992C8.60078 15.4992 1.80078 22.4992 1.80078 31.0992C1.80078 39.7992 8.80078 46.7992 17.4008 46.7992C18.6008 46.7992 19.7008 45.7992 19.7008 44.4992C19.7008 43.1992 18.7008 42.1992 17.4008 42.1992C11.3008 42.1992 6.30078 37.1992 6.30078 30.9992C6.30078 24.7992 11.3008 19.7992 17.4008 19.7992C17.7008 19.7992 18.0008 19.7992 18.3008 19.7992C19.3008 19.8992 20.1008 19.2992 20.5008 18.3992C22.3008 14.2992 26.3008 11.5992 30.7008 11.5992C36.8008 11.5992 41.8008 16.5992 41.8008 22.7992C41.8008 23.2992 41.8008 23.5992 41.7008 23.9992C41.6008 24.8992 41.9008 25.6992 42.7008 26.1992C43.4008 26.6992 44.4008 26.6992 45.1008 26.2992C46.4008 25.4992 47.9008 25.0992 49.4008 25.0992C54.0008 25.0992 57.8008 28.8992 57.8008 33.5992C57.8008 38.2992 54.0008 42.0992 49.4008 42.0992H47.3008C46.1008 42.0992 45.0008 43.0992 45.0008 44.3992C45.0008 45.6992 46.0008 46.6992 47.3008 46.6992H49.4008C56.5008 46.6992 62.3008 40.8992 62.3008 33.6992C62.3008 26.4992 56.5008 20.6992 49.3008 20.6992Z"/>
        <path d="M40.2016 48C41.3016 46.4 41.9016 44.5 41.9016 42.5C41.9016 39.9 40.9016 37.4 39.0016 35.6C35.2016 31.8 28.9016 31.8 25.0016 35.6C23.1016 37.4 22.1016 39.9 22.1016 42.5C22.1016 45.1 23.1016 47.6 25.0016 49.4C26.9016 51.3 29.5016 52.2 32.0016 52.2C33.7016 52.2 35.3016 51.8 36.8016 50.9L41.9016 55.9C42.3016 56.3 42.9016 56.5 43.5016 56.5C44.1016 56.5 44.7016 56.3 45.1016 55.8C46.0016 54.9 46.0016 53.5 45.1016 52.6L40.2016 48ZM28.2016 46.3C27.2016 45.3 26.6016 44 26.6016 42.6C26.6016 41.2 27.2016 39.9 28.2016 38.9C29.3016 37.9 30.6016 37.3 32.0016 37.3C33.4016 37.3 34.8016 37.8 35.8016 38.9C36.8016 39.9 37.4016 41.2 37.4016 42.6C37.4016 44 36.8016 45.3 35.8016 46.3C33.7016 48.3 30.3016 48.3 28.2016 46.3Z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_cross(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-cross">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M41.2,22.7c-0.9-0.9-2.3-0.9-3.2,0L32,28.8l-6.1-6.1c-0.9-0.9-2.3-0.9-3.2,0c-0.9,0.9-0.9,2.3,0,3.2l6.1,6.1l-6.1,6.1
            c-0.9,0.9-0.9,2.3,0,3.2c0.4,0.4,1,0.7,1.6,0.7c0.6,0,1.2-0.2,1.6-0.7l6.1-6.1l6.1,6.1c0.4,0.4,1,0.7,1.6,0.7
            c0.6,0,1.2-0.2,1.6-0.7c0.9-0.9,0.9-2.3,0-3.2L35.2,32l6.1-6.1C42.1,25,42.1,23.6,41.2,22.7z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_check(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-check">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M40.6,22.7L28.7,34.3L23.3,29c-0.9-0.9-2.3-0.8-3.2,0c-0.9,0.9-0.8,2.3,0,3.2l6.4,6.2c0.6,0.6,1.4,0.9,2.2,0.9
            c0.8,0,1.6-0.3,2.2-0.9L43.8,26c0.9-0.9,0.9-2.3,0-3.2S41.5,21.9,40.6,22.7z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_chevron_left(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-chevron-left">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M40.2,16.9c-0.9-0.9-2.3-0.9-3.2,0L23.8,30.4c-0.9,0.9-0.9,2.3,0,3.2L37,47.1c0.4,0.4,1,0.7,1.6,0.7c0.6,0,1.1-0.2,1.6-0.6
            c0.9-0.9,0.9-2.3,0-3.2L28.5,32l11.7-11.9C41.1,19.2,41.1,17.7,40.2,16.9z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_chevron_up(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-chevron-up">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M33.6,23.8c-0.9-0.9-2.3-0.9-3.2,0L16.9,37c-0.9,0.9-0.9,2.3,0,3.2c0.4,0.4,1,0.7,1.6,0.7c0.6,0,1.1-0.2,1.6-0.6L32,28.5
            l11.9,11.7c0.9,0.9,2.3,0.9,3.2,0c0.9-0.9,0.9-2.3,0-3.2L33.6,23.8z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_chevron_right(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-chevron-right">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M27,16.9c-0.9-0.9-2.3-0.9-3.2,0c-0.9,0.9-0.9,2.3,0,3.2L35.5,32L23.8,43.9c-0.9,0.9-0.9,2.3,0,3.2c0.4,0.4,1,0.6,1.6,0.6
            c0.6,0,1.2-0.2,1.6-0.7l13.3-13.5c0.9-0.9,0.9-2.3,0-3.2L27,16.9z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_chevron_down(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-chevron-down">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M43.9,23.8L32,35.5L20.1,23.8c-0.9-0.9-2.3-0.9-3.2,0c-0.9,0.9-0.9,2.3,0,3.2l13.5,13.3c0.4,0.4,1,0.6,1.6,0.6
            c0.6,0,1.1-0.2,1.6-0.6L47.1,27c0.9-0.9,0.9-2.3,0-3.2C46.3,22.9,44.8,22.9,43.9,23.8z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_hamburger(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-hamburger">
        <path class="opt" d="M32.0008 1.80078C15.3008 1.80078 1.80078 15.3008 1.80078 32.0008C1.80078 48.7008 15.3008 62.3008 32.0008 62.3008C48.7008 62.3008 62.3008 48.7008 62.3008 32.0008C62.3008 15.3008 48.7008 1.80078 32.0008 1.80078ZM32.0008 57.8008C17.8008 57.8008 6.30078 46.2008 6.30078 32.0008C6.30078 17.8008 17.8008 6.30078 32.0008 6.30078C46.2008 6.30078 57.8008 17.9008 57.8008 32.1008C57.8008 46.2008 46.2008 57.8008 32.0008 57.8008Z"/>
        <path d="M42.1016 18.1016H21.9016C20.7016 18.1016 19.6016 19.1016 19.6016 20.4016C19.6016 21.7016 20.6016 22.7016 21.9016 22.7016H42.0016C43.2016 22.7016 44.3016 21.7016 44.3016 20.4016C44.3016 19.1016 43.3016 18.1016 42.1016 18.1016Z"/>
        <path d="M42.1016 29.8008H21.9016C20.7016 29.8008 19.6016 30.8008 19.6016 32.1008C19.6016 33.3008 20.6016 34.4008 21.9016 34.4008H42.0016C43.2016 34.4008 44.3016 33.4008 44.3016 32.1008C44.3016 30.8008 43.3016 29.8008 42.1016 29.8008Z"/>
        <path d="M42.1016 41.4004H21.9016C20.7016 41.4004 19.6016 42.4004 19.6016 43.7004C19.6016 45.0004 20.6016 46.0004 21.9016 46.0004H42.0016C43.2016 46.0004 44.3016 45.0004 44.3016 43.7004C44.3016 42.4004 43.3016 41.4004 42.1016 41.4004Z"/>
    </svg>"#.to_string())?)
}

pub fn icon_circle_question(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-circle-question">
        <path class="opt" d="M32,1.8C15.3,1.8,1.8,15.3,1.8,32S15.3,62.3,32,62.3S62.3,48.7,62.3,32S48.7,1.8,32,1.8z M32,57.8
            C17.8,57.8,6.3,46.2,6.3,32C6.3,17.8,17.8,6.3,32,6.3c14.2,0,25.8,11.6,25.8,25.8C57.8,46.2,46.2,57.8,32,57.8z"/>
        <path d="M33.8,12.1c-2.9-0.5-5.9,0.3-8.1,2.2c-2.2,1.9-3.5,4.6-3.5,7.6c0,1.1,0.2,2.2,0.6,3.3c0.4,1.2,1.7,1.8,2.9,1.4
            c1.2-0.4,1.8-1.7,1.4-2.9c-0.2-0.6-0.3-1.2-0.3-1.8c0-1.6,0.7-3.1,1.9-4.1c1.2-1,2.8-1.5,4.5-1.2c2.1,0.4,3.9,2.2,4.3,4.3
            c0.4,2.5-0.9,5-3.2,6c-2.6,1.1-4.3,3.7-4.3,6.7v6.2c0,1.2,1,2.3,2.3,2.3c1.2,0,2.3-1,2.3-2.3v-6.2c0-1.1,0.6-2.1,1.5-2.5
            c4.3-1.8,6.8-6.3,6-10.9C41,16.1,37.8,12.8,33.8,12.1z"/>
        <path d="M32.1,45.8h-0.3c-1.2,0-2.3,1-2.3,2.3s1,2.3,2.3,2.3h0.3c1.2,0,2.2-1,2.2-2.3S33.4,45.8,32.1,45.8z"/>
    </svg>"#.to_string())?)
}

pub fn icon_mailbox(_args: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(to_value(r#"
    <svg viewBox="0 0 64 64" class="svg-icon icon-mailbox">
        <path d="M47.1,12.6V6.9c0-0.4,0.3-0.6,0.6-0.6h7.9c1.2,0,2.3-1,2.3-2.3s-1-2.3-2.3-2.3h-7.9c-2.8,0-5.1,2.3-5.1,5.1v5.4
            c0,0-23.1,0-23.1,0c-7.3,0-13.3,6-13.3,13.3v16.1c0,3.4,2.7,6.1,6.1,6.1h20.4V60c0,1.2,1,2.3,2.3,2.3s2.3-1,2.3-2.3V47.8h12
            c4.5,0,8.1-3.6,8.1-8.1V25.2c0-3.5-1.3-6.7-3.8-9.1C51.7,14.3,49.5,13.1,47.1,12.6z M12.3,43.3c-0.9,0-1.6-0.7-1.6-1.6V25.6
            c0-4.8,3.9-8.8,8.8-8.8c5,0,9.1,4.1,9.1,9.2v17.3H12.3z M52.8,39.7c0,2-1.6,3.6-3.6,3.6H33.1V25.9c0-3.5-1.4-6.7-3.5-9.1h13v9.4
            c0,1.2,1,2.3,2.3,2.3s2.3-1,2.3-2.3v-8.9c1.2,0.4,2.3,1.1,3.2,2c1.6,1.6,2.5,3.7,2.5,6V39.7z"/>
        <path class="opt" d="M21.6,34.1h-5.5c-1.2,0-2.3,1-2.3,2.3s1,2.3,2.3,2.3h5.5c1.2,0,2.3-1,2.3-2.3S22.9,34.1,21.6,34.1z"/>
    </svg>"#.to_string())?)
}