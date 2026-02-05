use crate::{DEFAULT_SIP_PORT, DEFAULT_SIPS_PORT};
use imersio_sip::{Host, SipUri, Transport, UriScheme};
use serde::Deserialize;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub(crate) proxy: ProxyConfig,
}

impl TryInto<Config> for &Path {
    type Error = String;

    fn try_into(self) -> Result<Config, Self::Error> {
        let content = std::fs::read_to_string(self)
            .map_err(|e| format!("Could not read config file '{}': {}", self.display(), e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Could not parse config file '{}': {}", self.display(), e))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProxyConfig {
    #[serde(default)]
    pub(crate) log_level: LogLevel,
    #[serde(default = "default_transports")]
    pub(crate) transports: HashSet<SipUri>,
}

fn default_transports() -> HashSet<SipUri> {
    HashSet::from([
        SipUri::try_from("sip:0.0.0.0:5060").unwrap(),
        SipUri::try_from("sip:[::]:5060").unwrap(),
    ])
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TransportsByIpType {
    pub(crate) ipv4: Option<SipUri>,
    pub(crate) ipv6: Option<SipUri>,
}

pub(crate) fn transports_by_ip_type<I>(transports: I) -> Vec<TransportsByIpType>
where
    I: IntoIterator<Item = SipUri>,
{
    fn sip_uri_matches(
        sip_uri: &SipUri,
        ip: IpAddr,
        port: Option<u16>,
        transport: &Option<Transport>,
    ) -> bool {
        sip_uri.host() == &Host::Ip(ip)
            && sip_uri.port() == port
            && &sip_uri.transport() == transport
    }

    fn insert_ipv4_transport(
        transports: &mut Vec<TransportsByIpType>,
        ipv6: Ipv6Addr,
        port: Option<u16>,
        sip_uri: &SipUri,
    ) {
        match transports.iter_mut().find(|t| {
            matches!(t.ipv6.as_ref(),
            Some(found_sip_uri)
                if sip_uri_matches(found_sip_uri, IpAddr::V6(ipv6), port, &sip_uri.transport()))
        }) {
            Some(transport_by_ip_type) => {
                transport_by_ip_type.ipv4 = Some(sip_uri.clone());
            }
            None => transports.push(TransportsByIpType {
                ipv4: Some(sip_uri.clone()),
                ipv6: None,
            }),
        }
    }

    fn insert_ipv6_transport(
        transports: &mut Vec<TransportsByIpType>,
        ipv4: Ipv4Addr,
        port: Option<u16>,
        sip_uri: &SipUri,
    ) {
        match transports.iter_mut().find(|t| {
            matches!(t.ipv4.as_ref(),
            Some(found_sip_uri)
                if sip_uri_matches(found_sip_uri, IpAddr::V4(ipv4), port, &sip_uri.transport()))
        }) {
            Some(transport_by_ip_type) => {
                transport_by_ip_type.ipv6 = Some(sip_uri.clone());
            }
            None => transports.push(TransportsByIpType {
                ipv4: None,
                ipv6: Some(sip_uri.clone()),
            }),
        }
    }

    fn insert_transport(transports: &mut Vec<TransportsByIpType>, transport: SipUri) {
        let port = transport.port();
        match transport.host() {
            Host::Ip(ip) => match ip {
                IpAddr::V4(ip) => {
                    if ip.is_unspecified() {
                        insert_ipv4_transport(transports, Ipv6Addr::UNSPECIFIED, port, &transport);
                    } else if ip.is_loopback() {
                        insert_ipv4_transport(transports, Ipv6Addr::LOCALHOST, port, &transport);
                    } else {
                        insert_ipv4_transport(transports, ip.to_ipv6_mapped(), port, &transport);
                    }
                }
                IpAddr::V6(ip) => {
                    if ip.is_unspecified() {
                        insert_ipv6_transport(transports, Ipv4Addr::UNSPECIFIED, port, &transport);
                    } else if ip.is_loopback() {
                        insert_ipv6_transport(transports, Ipv4Addr::LOCALHOST, port, &transport);
                    } else {
                        match ip.to_ipv4_mapped() {
                            Some(ipv4) => {
                                insert_ipv6_transport(transports, ipv4, port, &transport);
                            }
                            None => transports.push(TransportsByIpType {
                                ipv4: None,
                                ipv6: Some(transport.clone()),
                            }),
                        }
                    }
                }
            },
            Host::Name(_name) => {
                todo!()
            }
        }
    }

    let mut result = Vec::new();

    for sip_uri in transports {
        let port = sip_uri.port();
        match sip_uri.scheme() {
            UriScheme::Sip => match sip_uri.transport() {
                None => {
                    let mut builder = sip_uri.into_builder();
                    if port.is_none() {
                        builder.port(DEFAULT_SIP_PORT);
                    }
                    let udp_sip_uri = builder.clone().transport_parameter(Transport::Udp).build();
                    insert_transport(&mut result, udp_sip_uri);
                    let tcp_sip_uri = builder.transport_parameter(Transport::Tcp).build();
                    insert_transport(&mut result, tcp_sip_uri);
                }
                Some(_) => {
                    insert_transport(&mut result, sip_uri);
                }
            },
            UriScheme::Sips => {
                let mut builder = sip_uri.into_builder();
                if port.is_none() {
                    builder.port(DEFAULT_SIPS_PORT);
                }
                let sip_uri = builder.transport_parameter(Transport::Tls).build();
                insert_transport(&mut result, sip_uri);
            }
            _ => unreachable!(),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::config::transports_by_ip_type;
    use imersio_sip::SipUri;

    #[test]
    fn test_transports_by_ip_type_unspecified() {
        let transports: Vec<SipUri> = vec![
            "sip:0.0.0.0:5060".parse().unwrap(),
            "sip:[::]:5060".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            "sip:0.0.0.0:5060;transport=udp".parse().ok()
        );
        assert_eq!(
            transports[0].ipv6,
            "sip:[::]:5060;transport=udp".parse().ok()
        );
        assert_eq!(
            transports[1].ipv4,
            "sip:0.0.0.0:5060;transport=tcp".parse().ok()
        );
        assert_eq!(
            transports[1].ipv6,
            "sip:[::]:5060;transport=tcp".parse().ok()
        );
    }

    #[test]
    fn test_transports_by_ip_type_localhost_with_same_transport_parameter() {
        let transports: Vec<SipUri> = vec![
            "sip:127.0.0.1:5060;transport=tcp".parse().unwrap(),
            "sip:[::1]:5060;transport=tcp".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 1);
        assert_eq!(
            transports[0].ipv4,
            "sip:127.0.0.1:5060;transport=tcp".parse().ok()
        );
        assert_eq!(
            transports[0].ipv6,
            "sip:[::1]:5060;transport=tcp".parse().ok()
        );
    }

    #[test]
    fn test_transports_by_ip_type_localhost_with_different_transport_parameters() {
        let transports: Vec<SipUri> = vec![
            "sip:127.0.0.1:5060;transport=udp".parse().unwrap(),
            "sip:[::1]:5060;transport=tcp".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            "sip:127.0.0.1:5060;transport=udp".parse().ok()
        );
        assert_eq!(transports[0].ipv6, None);
        assert_eq!(transports[1].ipv4, None);
        assert_eq!(
            transports[1].ipv6,
            "sip:[::1]:5060;transport=tcp".parse().ok()
        );
    }

    #[test]
    fn test_transports_by_ip_type_ipv4_mapped() {
        let transports: Vec<SipUri> = vec![
            "sip:[::ffff:192.168.0.13]:5060".parse().unwrap(),
            "sip:192.168.0.13:5060".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            "sip:192.168.0.13:5060;transport=udp".parse().ok()
        );
        assert_eq!(
            transports[0].ipv6,
            "sip:[::ffff:192.168.0.13]:5060;transport=udp".parse().ok()
        );
        assert_eq!(
            transports[1].ipv4,
            "sip:192.168.0.13:5060;transport=tcp".parse().ok()
        );
        assert_eq!(
            transports[1].ipv6,
            "sip:[::ffff:192.168.0.13]:5060;transport=tcp".parse().ok()
        );
    }

    #[test]
    fn test_transports_by_ip_type_with_and_without_transport_parameter() {
        let transports: Vec<SipUri> = vec![
            "sip:127.0.0.1:5060".parse().unwrap(),
            "sip:[::1]:5060;transport=tcp".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            Some("sip:127.0.0.1:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(transports[0].ipv6, None);
        assert_eq!(
            transports[1].ipv4,
            Some("sip:127.0.0.1:5060;transport=tcp".parse().unwrap())
        );
        assert_eq!(
            transports[1].ipv6,
            Some("sip:[::1]:5060;transport=tcp".parse().unwrap())
        );
    }

    #[test]
    fn test_transports_by_ip_type_sips() {
        let transports: Vec<SipUri> = vec![
            "sips:127.0.0.1:5061".parse().unwrap(),
            "sips:[::1]:5061".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 1);
        assert_eq!(
            transports[0].ipv4,
            Some("sips:127.0.0.1:5061;transport=tls".parse().unwrap())
        );
        assert_eq!(
            transports[0].ipv6,
            Some("sips:[::1]:5061;transport=tls".parse().unwrap())
        );
    }

    #[test]
    fn test_transports_by_ip_type_sip_and_sips_uris() {
        let transports: Vec<SipUri> = vec![
            "sip:0.0.0.0:5060".parse().unwrap(),
            "sips:[::]:5061".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 3);
        assert_eq!(
            transports[0].ipv4,
            Some("sip:0.0.0.0:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(transports[0].ipv6, None);
        assert_eq!(
            transports[1].ipv4,
            Some("sip:0.0.0.0:5060;transport=tcp".parse().unwrap())
        );
        assert_eq!(transports[1].ipv6, None);
        assert_eq!(transports[2].ipv4, None);
        assert_eq!(
            transports[2].ipv6,
            Some("sips:[::]:5061;transport=tls".parse().unwrap())
        );
    }

    #[test]
    fn test_transports_by_ip_type_without_ports() {
        let transports: Vec<SipUri> = vec![
            "sip:127.0.0.1".parse().unwrap(),
            "sip:[::1]".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            Some("sip:127.0.0.1:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(
            transports[0].ipv6,
            Some("sip:[::1]:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(
            transports[1].ipv4,
            Some("sip:127.0.0.1:5060;transport=tcp".parse().unwrap())
        );
        assert_eq!(
            transports[1].ipv6,
            Some("sip:[::1]:5060;transport=tcp".parse().unwrap())
        );
    }

    #[test]
    fn test_transports_by_ip_type_with_and_without_ports_standard() {
        let transports: Vec<SipUri> = vec![
            "sip:0.0.0.0:5060".parse().unwrap(),
            "sip:[::]".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 2);
        assert_eq!(
            transports[0].ipv4,
            Some("sip:0.0.0.0:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(
            transports[0].ipv6,
            Some("sip:[::]:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(
            transports[1].ipv4,
            Some("sip:0.0.0.0:5060;transport=tcp".parse().unwrap())
        );
        assert_eq!(
            transports[1].ipv6,
            Some("sip:[::]:5060;transport=tcp".parse().unwrap())
        );
    }

    #[test]
    fn test_transports_by_ip_type_with_and_without_ports_ipv4_other() {
        let transports: Vec<SipUri> = vec![
            "sip:192.168.0.13".parse().unwrap(),
            "sip:[::ffff:192.168.0.13]:5080".parse().unwrap(),
        ];

        let transports = transports_by_ip_type(transports);
        assert_eq!(transports.len(), 4);
        assert_eq!(
            transports[0].ipv4,
            Some("sip:192.168.0.13:5060;transport=udp".parse().unwrap())
        );
        assert_eq!(transports[0].ipv6, None);
        assert_eq!(
            transports[1].ipv4,
            Some("sip:192.168.0.13:5060;transport=tcp".parse().unwrap())
        );
        assert_eq!(transports[1].ipv6, None);
        assert_eq!(transports[2].ipv4, None);
        assert_eq!(
            transports[2].ipv6,
            Some(
                "sip:[::ffff:192.168.0.13]:5080;transport=udp"
                    .parse()
                    .unwrap()
            )
        );
        assert_eq!(transports[3].ipv4, None);
        assert_eq!(
            transports[3].ipv6,
            Some(
                "sip:[::ffff:192.168.0.13]:5080;transport=tcp"
                    .parse()
                    .unwrap()
            )
        );
    }
}
