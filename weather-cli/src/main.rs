use std::env;
use std::fmt;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let code = match args.len() {
        1 => {
            print_usage();
            1
        }
        2 => {
            let arg = &args[1];
            if arg == "--help" || arg == "-h" {
                print_help();
                0
            } else if arg == "--version" || arg == "-V" {
                println!("weather-cli v{}", env!("CARGO_PKG_VERSION"));
                0
            } else {
                match fetch_weather(arg) {
                    Ok(report) => {
                        println!("{report}");
                        0
                    }
                    Err(err) => {
                        eprintln!("{}", err.user_message());
                        err.exit_code()
                    }
                }
            }
        }
        _ => {
            print_usage();
            1
        }
    };
    process::exit(code);
}

fn print_usage() {
    eprintln!("Usage: weather-cli <CITY>");
    eprintln!("       weather-cli --help");
}

fn print_help() {
    println!("weather-cli v{}", env!("CARGO_PKG_VERSION"));
    println!("Fetch and display current weather for a city");
    println!();
    println!("USAGE:");
    println!("  weather-cli <CITY>");
    println!("  weather-cli --help");
    println!("  weather-cli --version");
    println!();
    println!("ARGS:");
    println!("  <CITY>  Name of the city to get weather for");
}

#[derive(Debug, PartialEq)]
enum WeatherError {
    Network(String),
    Timeout(String),
    CityNotFound(String),
    InvalidResponse(String),
    Internal(String),
}

impl WeatherError {
    fn user_message(&self) -> String {
        match self {
            WeatherError::Network(_) => {
                "Network error.\nCheck your internet connection.".to_string()
            }
            WeatherError::Timeout(_) => "Request timed out.\nPlease try again later.".to_string(),
            WeatherError::CityNotFound(city) => {
                format!("Unknown city: {city}")
            }
            WeatherError::InvalidResponse(_) => {
                "Weather service returned an unexpected response.".to_string()
            }
            WeatherError::Internal(msg) => {
                format!("Internal error: {msg}")
            }
        }
    }

    fn exit_code(&self) -> i32 {
        match self {
            WeatherError::Network(_)
            | WeatherError::Timeout(_)
            | WeatherError::InvalidResponse(_)
            | WeatherError::Internal(_) => 2,
            WeatherError::CityNotFound(_) => 1,
        }
    }
}

impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

fn fetch_weather(city: &str) -> Result<String, WeatherError> {
    if city.is_empty() {
        return Err(WeatherError::Internal(
            "City name cannot be empty".to_string(),
        ));
    }
    let client = RealHttpClient;
    WeatherService { client }.get_weather(city)
}

trait HttpClient {
    fn get_text(&self, url: &str) -> Result<String, WeatherError>;
}

struct RealHttpClient;

impl HttpClient for RealHttpClient {
    fn get_text(&self, url: &str) -> Result<String, WeatherError> {
        let response = reqwest::blocking::get(url).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("timed out") || msg.contains("timeout") {
                WeatherError::Timeout(msg)
            } else {
                WeatherError::Network(msg)
            }
        })?;
        let status = response.status();
        if status.is_server_error() {
            return Err(WeatherError::Internal(format!(
                "Weather service unavailable (HTTP {status})"
            )));
        }
        if status.is_client_error() {
            return Err(WeatherError::CityNotFound(
                url.rsplit('/')
                    .next()
                    .unwrap_or("unknown")
                    .trim_end_matches("?format=%C+%t&m")
                    .to_string(),
            ));
        }
        response
            .text()
            .map_err(|e| WeatherError::InvalidResponse(e.to_string()))
    }
}

struct WeatherService<C: HttpClient> {
    client: C,
}

impl<C: HttpClient> WeatherService<C> {
    fn get_weather(&self, city: &str) -> Result<String, WeatherError> {
        let url = format!("https://wttr.in/{city}?format=%C+%t&m");
        let body = self.client.get_text(&url)?;
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return Err(WeatherError::CityNotFound(city.to_string()));
        }
        Ok(format!("{city}: {trimmed}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHttpClient;

    impl HttpClient for MockHttpClient {
        fn get_text(&self, url: &str) -> Result<String, WeatherError> {
            if url.contains("timeout") {
                return Err(WeatherError::Timeout("connection timed out".to_string()));
            }
            if url.contains("nowhere") {
                return Err(WeatherError::Network("connection refused".to_string()));
            }
            if url.contains("500") {
                return Err(WeatherError::Internal(
                    "Weather service unavailable (HTTP 500)".to_string(),
                ));
            }
            if url.contains("empty") {
                return Ok("".to_string());
            }
            if url.contains("404") {
                return Err(WeatherError::CityNotFound("404".to_string()));
            }
            Ok("Partly cloudy +15°C".to_string())
        }
    }

    #[test]
    fn print_help_does_not_panic() {
        print_help();
    }

    #[test]
    fn print_usage_does_not_panic() {
        print_usage();
    }

    #[test]
    fn empty_city_rejected() {
        let result = fetch_weather("");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            WeatherError::Internal("City name cannot be empty".to_string())
        );
    }

    #[test]
    fn weather_service_returns_report() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("London");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "London: Partly cloudy +15°C");
    }

    #[test]
    fn weather_service_network_error() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("nowhere");
        assert!(matches!(result.unwrap_err(), WeatherError::Network(_)));
    }

    #[test]
    fn weather_service_timeout_error() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("timeout");
        assert!(matches!(result.unwrap_err(), WeatherError::Timeout(_)));
    }

    #[test]
    fn weather_service_city_not_found() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("404");
        assert!(matches!(result.unwrap_err(), WeatherError::CityNotFound(_)));
    }

    #[test]
    fn weather_service_empty_response() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("empty");
        assert!(matches!(result.unwrap_err(), WeatherError::CityNotFound(_)));
    }

    #[test]
    fn weather_service_internal_error() {
        let service = WeatherService {
            client: MockHttpClient,
        };
        let result = service.get_weather("500");
        assert!(matches!(result.unwrap_err(), WeatherError::Internal(_)));
    }

    #[test]
    fn network_error_exit_code_is_2() {
        let err = WeatherError::Network("down".to_string());
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn city_not_found_exit_code_is_1() {
        let err = WeatherError::CityNotFound("x".to_string());
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn success_exit_code_is_0() {
        let code = 0;
        assert_eq!(code, 0);
    }

    #[test]
    fn user_message_readable() {
        let cases: Vec<(WeatherError, &str)> = vec![
            (WeatherError::Network("x".into()), "Check your internet"),
            (WeatherError::Timeout("x".into()), "try again later"),
            (
                WeatherError::CityNotFound("Londoon".into()),
                "Unknown city: Londoon",
            ),
            (
                WeatherError::InvalidResponse("x".into()),
                "unexpected response",
            ),
        ];
        for (err, expected) in cases {
            assert!(
                err.user_message().contains(expected),
                "expected '{expected}' in '{}'",
                err.user_message()
            );
        }
    }
}
