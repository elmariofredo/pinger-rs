use std::io::{Error};
use std::thread;

use serde_yaml::Value;
use schedule_recv::{periodic_ms};
use prometheus::{Opts, Registry, CounterVec};
use chrono::prelude::*;
use curl::easy::Easy;

pub struct Metrics {
    pub registry: Registry,
    config: Value,
}

impl Metrics {

  pub fn new(conf: Value) -> Result<Metrics, Error>{

    let registry = Registry::new();
    let config = conf;
    Ok(Metrics { registry, config,})
  }

  fn polling(&self, counter_vec: CounterVec, url_serde: Value) {

    let interval = self.config["pinger"]["interval"] 
      .as_str().unwrap_or("10");
    let interval_ms: u32 = interval.parse().unwrap_or(10) * 1000;


    let debug = self.config["pinger"]["debug"] 
      .as_bool().unwrap_or(true);

    thread::spawn(move || {

      let delay = periodic_ms(interval_ms);
      let mut easy = Easy::new();


      let url = url_serde.as_str()
        .expect("URL must be set");

      println!("Polling {} every {} seconds", url, interval_ms / 1000);
      println!("Debug is set to {}", debug);

      loop {

        delay.recv().unwrap();

        easy.url(url).unwrap();
        easy.write_function(|data| {
          Ok(data.len())
        }).unwrap();
        
        let dt = Local::now();

        match easy.perform() {
          Ok(_) => {

            let code = easy.response_code().unwrap().to_string();
            counter_vec.with_label_values(&[&code, url]).inc();
            
            if debug {
              println!("{}: {} - {}", dt, code, url);
            }
          },
          Err(_) => println!("{}: Error accessing {}", dt, url),
        }
      }
    });

  }

  pub fn init(&self) {

    // Create a Counter.
    let metrics_name = self.config["pinger"]["metric-name"] 
      .as_str().unwrap_or("pinger_metrics");

    let counter_opts = Opts::new(metrics_name, "test counter help");
    let counter = CounterVec::new(counter_opts, &["code", "url"]).unwrap();

    // Register Counter
    self.registry.register(Box::new(counter.clone())).unwrap();

    let urls: Vec<Value> = self.config["pinger"]["hosts"]
      .as_sequence().unwrap().to_vec();

    for url in urls {
      self.polling(counter.clone(), url);
    }
  }
}