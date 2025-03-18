use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use serde_json::json;

use kube::{
    api::{Api, ListParams, PostParams, ResourceExt, WatchEvent},
    Client,
};

pub async fn create_jupyter_pod(namespace: String, image: String) -> anyhow::Result<()> {
    let client = Client::try_default().await?;

    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    let p: Pod = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "generateName": "dracula-jupyter-py-",
            "annotations": {"iam.amazonaws.com/role": "k8s-jupyter" }
        },
        "spec": {
            "restartPolicy": "Never",
            "containers": [
              {
                "command": [
                  "bash",
                  "-c",
                  "sleep 50; python /usr/bin/materialize_views.py"
                ],
                "image": image,
                "imagePullPolicy": "Always",
                "name": "jupyter-py"
              }
            ]
        }
    }))?;

    let pp = PostParams::default();
    match pods.create(&pp, &p).await {
        Ok(o) => {
            let name = o.name();
            assert_eq!(p.name(), name);
            println!("Created {}", name);
            // wait for it..
            std::thread::sleep(std::time::Duration::from_millis(5_000));
        }
        Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
        Err(e) => return Err(e.into()),                        // any other case is probably bad
    }

    let lp = ListParams::default()
        .fields(&format!("metadata.name={}", "jupyter-py"))
        .timeout(10);
    let mut stream = pods.watch(&lp, "0").await?.boxed();
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(o) => println!("Added {}", o.name()),
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                let phase = s.phase.clone().unwrap_or_default();
                println!("Modified: {} with phase: {}", o.name(), phase);
            }
            WatchEvent::Deleted(o) => println!("Deleted {}", o.name()),
            WatchEvent::Error(e) => println!("Error {}", e),
            _ => {}
        }
    }

    Ok(())
}
