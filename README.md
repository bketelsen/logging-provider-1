# waSCC Logging Provider
This library is a _native capability provider_ for the `wascc:logging` capability. Only actors signed with tokens containing this capability privilege will be allowed to use it. 

It should be compiled as a native linux (`.so`) binary and made available to the **waSCC** host runtime as a plugin. 

## Usage

Inside a _wascc_ actor, use normal `log` macros to write logs:

```
#[macro_use]
extern crate log;

fn hello_world(
   ctx: &CapabilitiesContext,
   r: http::Request) -> CallResult {
    // regular log macros get intercepted automatically
    // and translated into waPC calls to the wascc:logging 
    // provider
    warn!("warn something");
    info!("info something");

    // ctx.println logs to the hosts logging context
    ctx.println(&format!(" Received HTTP request: {:?}", &r));
    
    let echo = EchoRequest {
        method: r.method,
        path: r.path,
        query_string: r.query_string,
        body: r.body,
    };
   
    // if you don't want to use the macros
    // you can get the log() and call
    // methods on it
    ctx.log().error("error body").unwrap();

    // or you can create a log request manually
    let l = WriteLogRequest {
        level: actor::logger::WARN, 
        body: "raw msg I'm a Body!".to_string(), 
    };
    // then send it
    ctx.raw().call("wascc:logging", "WriteLog", &wascc_codec::serialize(l)?)?;

    let resp = http::Response::json(echo, 200, "OK");

    Ok(serialize(resp)?)
}
```