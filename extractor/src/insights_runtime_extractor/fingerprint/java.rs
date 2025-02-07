use log::debug;

use super::FingerPrint;
use crate::insights_runtime_extractor::Config;
use crate::insights_runtime_extractor::ContainerProcess;

pub struct Java {}

impl Java {
    /// Check if the Java process is started with a jboss.home.dir system property
    fn jboss_modules_executable(
        out_dir: &String,
        process: &ContainerProcess,
    ) -> Option<Vec<String>> {
        debug!(
            "Process {} is using JBoss Modules with command line {:#?}",
            &process.pid, &process.command_line
        );

        return process
            .command_line
            .iter()
            .position(|s| s.starts_with("-Djboss.home.dir"))
            .and_then(|i| process.command_line.get(i))
            .and_then(|jboss_home_dir_sys_prop| jboss_home_dir_sys_prop.split_once("="))
            .and_then(|(_, jboss_home_dir)| {
                debug!(
                    "Process {} is using JBoss Module from JBoss Home {:#?}",
                    &process.pid, jboss_home_dir
                );

                Some(vec![
                    String::from("./fpr_java_jboss_modules"),
                    out_dir.to_string(),
                    jboss_home_dir.to_string(),
                ])
            });
    }

    fn jar_executable(
        out_dir: &String,
        process: &ContainerProcess,
        jar: &str,
    ) -> Option<Vec<String>> {
        let jar = match jar {
            jar if jar.starts_with("/") => jar.to_owned(),
            jar => format!("{:?}/{}", process.cwd, jar),
        };

        return Some(vec![
            String::from("./fpr_java_runtimes"),
            out_dir.to_string(),
            jar.to_string(),
        ]);
    }
}

impl FingerPrint for Java {
    fn can_apply_to(
        &self,
        _config: &Config,
        out_dir: &String,
        process: &ContainerProcess,
    ) -> Option<Vec<String>> {
        if !process.name.ends_with("java") {
            return None;
        }

        debug!("Fingerprint Java application from process: {:#?}", process);

        // check "java -jar" process
        let exec = process
            .command_line
            .iter()
            .position(|s| s == "-jar")
            .and_then(|i| process.command_line.get(i + 1))
            .and_then(|jar| {
                debug!("Executable jar is {:?}", jar);
                if jar.ends_with("jboss-modules.jar") {
                    return Java::jboss_modules_executable(&out_dir, process);
                } else {
                    return Java::jar_executable(&out_dir, process, jar);
                }
            });

        if exec.is_some() {
            return exec;
        }

        // check for Java classpath-based process
        process
            .command_line
            .iter()
            .position(|s| s == "-classpath" || s == "-cp")
            .and_then(|i| process.command_line.get(i + 1))
            .and_then(|classpath| {
                debug!("java process is using classpath {}", classpath);
                let mut jars = classpath.split(":");

                // let's find the corresponding jar based on the main class
                for java_fingerprints_config in _config.fingerprints.java.iter() {
                    if process
                        .command_line
                        .contains(&java_fingerprints_config.main_class)
                    {
                        debug!("Detected {} ", java_fingerprints_config.runtime_name);
                        let found = jars
                            .find(|jar| {
                                let main_jar = java_fingerprints_config.main_jar.as_ref().unwrap();
                                jar.contains(main_jar)
                            })
                            .and_then(|jar| return Java::jar_executable(&out_dir, process, &jar));
                        if found.is_some() {
                            return found.into();
                        }
                    }
                }
                None
            })
    }
}
