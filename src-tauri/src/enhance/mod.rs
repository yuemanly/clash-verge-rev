mod chain;
pub mod field;
mod merge;
mod script;
mod tun;

use self::chain::*;
use self::field::*;
use self::merge::*;
use self::script::*;
use self::tun::*;
use crate::config::Config;
use serde_yaml::Mapping;
use std::collections::HashMap;
use std::collections::HashSet;

type ResultLog = Vec<(String, String)>;

/// Enhance mode
/// 返回最终订阅、该订阅包含的键、和script执行的结果
pub fn enhance() -> (Mapping, Vec<String>, HashMap<String, ResultLog>) {
    // config.yaml 的订阅
    let clash_config = { Config::clash().latest().0.clone() };

    let (clash_core, enable_tun, enable_builtin, socks_enabled, http_enabled) = {
        let verge = Config::verge();
        let verge = verge.latest();
        (
            verge.clash_core.clone(),
            verge.enable_tun_mode.unwrap_or(false),
            verge.enable_builtin_enhanced.unwrap_or(true),
            verge.verge_socks_enabled.unwrap_or(true),
            verge.verge_http_enabled.unwrap_or(true),
        )
    };
    #[cfg(not(target_os = "windows"))]
    let redir_enabled = {
        let verge = Config::verge();
        let verge = verge.latest();
        verge.verge_redir_enabled.unwrap_or(true)
    };
    #[cfg(target_os = "linux")]
    let tproxy_enabled = {
        let verge = Config::verge();
        let verge = verge.latest();
        verge.verge_tproxy_enabled.unwrap_or(true)
    };

    // 从profiles里拿东西
    let (mut config, chain) = {
        let profiles = Config::profiles();
        let profiles = profiles.latest();

        let current = profiles.current_mapping().unwrap_or_default();

        let chain = match profiles.chain.as_ref() {
            Some(chain) => chain
                .iter()
                .filter_map(|uid| profiles.get_item(uid).ok())
                .filter_map(<Option<ChainItem>>::from)
                .collect::<Vec<ChainItem>>(),
            None => vec![],
        };

        (current, chain)
    };

    let mut result_map = HashMap::new(); // 保存脚本日志
    let mut exists_keys = use_keys(&config); // 保存出现过的keys

    // 处理用户的profile
    chain.into_iter().for_each(|item| match item.data {
        ChainType::Merge(merge) => {
            exists_keys.extend(use_keys(&merge));
            config = use_merge(merge, config.to_owned());
        }
        ChainType::Script(script) => {
            let mut logs = vec![];

            match use_script(script, config.to_owned()) {
                Ok((res_config, res_logs)) => {
                    exists_keys.extend(use_keys(&res_config));
                    config = res_config;
                    logs.extend(res_logs);
                }
                Err(err) => logs.push(("exception".into(), err.to_string())),
            }

            result_map.insert(item.uid, logs);
        }
    });

    // 合并默认的config
    for (key, value) in clash_config.into_iter() {
        if key.as_str() == Some("tun") {
            let mut tun = config.get_mut("tun").map_or(Mapping::new(), |val| {
                val.as_mapping().cloned().unwrap_or(Mapping::new())
            });
            let patch_tun = value.as_mapping().cloned().unwrap_or(Mapping::new());
            for (key, value) in patch_tun.into_iter() {
                tun.insert(key, value);
            }
            config.insert("tun".into(), tun.into());
        } else {
            if key.as_str() == Some("socks-port") && !socks_enabled {
                config.remove("socks-port");
                continue;
            }
            if key.as_str() == Some("port") && !http_enabled {
                config.remove("port");
                continue;
            }
            #[cfg(not(target_os = "windows"))]
            {
                if key.as_str() == Some("redir-port") && !redir_enabled {
                    config.remove("redir-port");
                    continue;
                }
            }
            #[cfg(target_os = "linux")]
            {
                if key.as_str() == Some("tproxy-port") && !tproxy_enabled {
                    config.remove("tproxy-port");
                    continue;
                }
            }
            config.insert(key, value);
        }
    }

    // 内建脚本最后跑
    if enable_builtin {
        ChainItem::builtin()
            .into_iter()
            .filter(|(s, _)| s.is_support(clash_core.as_ref()))
            .map(|(_, c)| c)
            .for_each(|item| {
                log::debug!(target: "app", "run builtin script {}", item.uid);

                match item.data {
                    ChainType::Script(script) => match use_script(script, config.to_owned()) {
                        Ok((res_config, _)) => {
                            config = res_config;
                        }
                        Err(err) => {
                            log::error!(target: "app", "builtin script error `{err}`");
                        }
                    },
                    _ => {}
                }
            });
    }

    config = use_tun(config, enable_tun);
    config = use_sort(config);

    let mut exists_set = HashSet::new();
    exists_set.extend(exists_keys.into_iter());
    exists_keys = exists_set.into_iter().collect();

    (config, exists_keys, result_map)
}
