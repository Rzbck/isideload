use idevice::{
    IdeviceService, RsdService,
    afc::AfcClient,
    installation_proxy::InstallationProxyClient,
    lockdown::LockdownClient,
    provider::{IdeviceProvider, RsdProvider},
    rsd::RsdHandshake,
};
use plist_macro::plist;
use rootcause::option_ext::OptionExt;
use rootcause::prelude::*;

use crate::SideloadError as Error;
use std::pin::Pin;
use std::{future::Future, path::Path};

/// Installs an ***already signed*** app onto your device.
/// To sign and install an app, see [`crate::sideload::sideload_app`]
pub async fn install_app(
    provider: &impl IdeviceProvider,
    app_path: &Path,
    progress_callback: impl Fn(u64) + Send + Sync,
) -> Result<(), Report> {
    // Keep one authenticated Lockdown session alive for the full AFC upload and
    // installation_proxy transaction. Direct Wi-Fi service sockets can be torn down
    // when the Lockdown session that started them is dropped, even though the same
    // pattern is tolerated over usbmux/USB.
    let pairing_file = provider
        .get_pairing_file()
        .await
        .map_err(Error::IdeviceError)?;
    let mut lockdown = LockdownClient::connect(provider)
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to connect to lockdown for app installation")?;
    let legacy = lockdown
        .start_session(&pairing_file)
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to start lockdown session for app installation")?;

    let (afc_port, afc_ssl) = lockdown
        .start_service(AfcClient::service_name())
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to start AFC service")?;
    let mut afc_idevice = provider
        .connect(afc_port)
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to connect to AFC service")?;
    if afc_ssl {
        afc_idevice
            .start_session(&pairing_file, legacy)
            .await
            .map_err(Error::IdeviceError)
            .context("Failed to secure AFC service connection")?;
    }
    let mut afc_client = AfcClient::from_stream(afc_idevice)
        .await
        .map_err(Error::IdeviceError)?;

    let dir = format!(
        "PublicStaging/{}",
        app_path.file_name().ok_or_report()?.to_string_lossy()
    );
    let total_size = get_dir_size(app_path).unwrap_or(1) as f64;
    let mut uploaded = 0;
    let cb = |progress: f64| {
        progress_callback((progress * 70.0) as u64);
    };
    afc_upload_dir(
        &mut afc_client,
        app_path,
        &dir,
        &cb,
        &mut uploaded,
        total_size,
    )
    .await?;

    let (instproxy_port, instproxy_ssl) = lockdown
        .start_service(InstallationProxyClient::service_name())
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to start installation proxy service")?;
    let mut instproxy_idevice = provider
        .connect(instproxy_port)
        .await
        .map_err(Error::IdeviceError)
        .context("Failed to connect to installation proxy service")?;
    if instproxy_ssl {
        instproxy_idevice
            .start_session(&pairing_file, legacy)
            .await
            .map_err(Error::IdeviceError)
            .context("Failed to secure installation proxy connection")?;
    }
    let mut instproxy_client = InstallationProxyClient::from_stream(instproxy_idevice)
        .await
        .map_err(Error::IdeviceError)?;

    let options = plist!(dict {
        "PackageType": "Developer"
    });

    instproxy_client
        .install_with_callback(
            dir,
            Some(plist::Value::Dictionary(options)),
            async |(percentage, _)| {
                progress_callback((70.0 + 0.3 * percentage as f64) as u64);
            },
            (),
        )
        .await
        .map_err(Error::IdeviceError)?;

    Ok(())
}

/// Installs an ***already signed*** app onto your device.
/// To sign and install an app, see [`crate::sideload::sideload_app`]
pub async fn install_app_rsd(
    provider: &mut impl RsdProvider,
    handshake: &mut RsdHandshake,
    app_path: &Path,
    progress_callback: impl Fn(u64) + Send + Sync,
) -> Result<(), Report> {
    let mut afc_client = AfcClient::connect_rsd(provider, handshake)
        .await
        .map_err(Error::IdeviceError)?;

    let dir = format!(
        "PublicStaging/{}",
        app_path.file_name().ok_or_report()?.to_string_lossy()
    );
    let total_size = get_dir_size(app_path).unwrap_or(1) as f64;
    let mut uploaded = 0;
    let cb = |pct: f64| {
        progress_callback((pct * 70.0) as u64);
    };
    afc_upload_dir(
        &mut afc_client,
        app_path,
        &dir,
        &cb,
        &mut uploaded,
        total_size,
    )
    .await?;

    let mut instproxy_client = InstallationProxyClient::connect_rsd(provider, handshake)
        .await
        .map_err(Error::IdeviceError)?;

    let options = plist!(dict {
        "PackageType": "Developer"
    });

    instproxy_client
        .install_with_callback(
            dir,
            Some(plist::Value::Dictionary(options)),
            async |(percentage, _)| {
                progress_callback((70.0 + 0.3 * percentage as f64) as u64);
            },
            (),
        )
        .await
        .map_err(Error::IdeviceError)?;

    Ok(())
}

fn get_dir_size(path: &Path) -> Result<u64, Report> {
    let mut size = 0;
    for entry in isideload_vfs::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let meta = isideload_vfs::fs::metadata(&path)?;
        if meta.is_dir() {
            size += get_dir_size(&path)?;
        } else {
            size += meta.len();
        }
    }
    Ok(size)
}

fn afc_upload_dir<'a>(
    afc_client: &'a mut AfcClient,
    path: &'a Path,
    afc_path: &'a str,
    cb: &'a (dyn Fn(f64) + Send + Sync),
    uploaded: &'a mut u64,
    total: f64,
) -> Pin<Box<dyn Future<Output = Result<(), Report>> + Send + 'a>> {
    Box::pin(async move {
        let entries = isideload_vfs::fs::read_dir(path)?;
        afc_client
            .mk_dir(afc_path)
            .await
            .map_err(Error::IdeviceError)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if isideload_vfs::fs::metadata(&path)?.is_dir() {
                let new_afc_path = format!(
                    "{}/{}",
                    afc_path,
                    path.file_name().ok_or_report()?.to_string_lossy()
                );
                afc_upload_dir(afc_client, &path, &new_afc_path, cb, uploaded, total).await?;
            } else {
                let mut file_handle = afc_client
                    .open(
                        format!(
                            "{}/{}",
                            afc_path,
                            path.file_name().ok_or_report()?.to_string_lossy()
                        ),
                        idevice::afc::opcode::AfcFopenMode::WrOnly,
                    )
                    .await
                    .map_err(Error::IdeviceError)?;

                let bytes = isideload_vfs::fs::read(&path)?;
                for chunk in bytes.chunks(8 * 1024) {
                    file_handle
                        .write_entire(chunk)
                        .await
                        .map_err(Error::IdeviceError)?;
                    *uploaded += chunk.len() as u64;
                    cb(*uploaded as f64 / total);
                }
                file_handle.close().await.map_err(Error::IdeviceError)?;
            }
        }
        Ok(())
    })
}
