use crate::config::cli_config::{write_config, CliConfig};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, ErrorKind},
    os::unix::process::CommandExt,
    process::Command,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Cluster {
    name: String,
    url: String,
    username: String,
}

/**
 * Adds or updates a cluster in the config file
 *  - If the cluster doesn't exist, it adds it
 *  - If the cluster already exists (checks the name), updates it's url and username
 */
pub fn add_cluster(
    name: String,
    url: String,
    username: String,
    mut cli_config: CliConfig,
) -> Result<(), io::Error> {
    let cluster_exists = find_cluster(&name, &mut cli_config.clusters);

    if cluster_exists.is_some() {
        let cluster = cluster_exists.unwrap();

        cluster.url = url;

        cluster.username = username;
    } else {
        let cluster = Cluster {
            name: name.clone(),
            url,
            username,
        };

        cli_config.clusters.push(cluster);
    }
    cli_config = write_config(cli_config)?;

    connect_to_cluster(name, cli_config)?;

    Ok(())
}

/**
 * Fetches the cluster from the config file and connects to it
 */
pub fn connect_to_cluster(
    cluster_name: String,
    mut cli_config: CliConfig,
) -> Result<(), io::Error> {
    let cluster = find_cluster(&cluster_name, &mut cli_config.clusters);

    if cluster.is_none() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!(
                "Cluster with name {} not found in config file, consider adding arguments --username and --cluster-url to save it",
                cluster_name
            ),
        ));
    }

    let cluster = cluster.unwrap();

    connect_command(cluster);

    Ok(())
}

/**
 * Lists all clusters
 */
pub fn list_clusters(wide: bool, cli_config: CliConfig) -> Result<(), io::Error> {
    let clusters = cli_config.clusters;

    clusters.iter().for_each(|c| {
        println!("{}", format_clusters_print(wide, c));
    });

    Ok(())
}

fn format_clusters_print(wide: bool, cluster: &Cluster) -> String {
    if wide == true {
        format!("{}\t{}\t{}", cluster.name, cluster.username, cluster.url)
    } else {
        format!("{}", cluster.name)
    }
}

fn find_cluster<'a>(name: &str, clusters: &'a mut Vec<Cluster>) -> Option<&'a mut Cluster> {
    clusters.iter_mut().find(|c| c.name.eq(name))
}

/**
 * Command to connect to the cluster
 * Uses `oc login`, perhaps use another method (login through rest api instead) ?
 */
fn connect_command(cluster: &Cluster) {
    Command::new("oc")
        .arg("login")
        .arg(&cluster.url)
        .arg("-u")
        .arg(&cluster.username)
        .exec();
}