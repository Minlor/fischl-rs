#[cfg(target_os = "linux")]
pub mod compat;
pub mod utils;
pub mod download;

#[cfg(test)]
mod tests {
    /*#[test]
    fn prefix_new() {
        let wp = "/home/tukan/.local/share/com.keqinglauncher.app/compatibility/runners/8.26-wine-ge-proton/bin/wine64".to_string();
        let pp = "/home/tukan/.local/share/com.keqinglauncher.app/compatibility/prefixes/nap_global/1.6.0".to_string();
        let prefix = Compat::setup_prefix(wp, pp);
        if prefix.is_ok() {
            let p = prefix.unwrap();
            Compat::end_session(p.wine.wineloader().to_str().unwrap().to_string(), p.wine.prefix.to_str().unwrap().to_string()).unwrap();
            Compat::shutdown(p.wine.wineloader().to_str().unwrap().to_string(), p.wine.prefix.to_str().unwrap().to_string()).unwrap();
            println!("Created prefix without issues!");
        } else {
            println!("Failed to create a prefix!");
        }
    }*/
    use std::path::Path;
    use crate::download::{Extras};
    use crate::download::game::{Game, Hoyo, Kuro, Sophon};
    use crate::utils::{extract_archive, prettify_bytes};
    use crate::utils::game::VoiceLocale;

    #[test]
    fn download_xxmi_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/extras/xxmi/testing";
        let success = Extras::download_xxmi(String::from("SpectrumQT/XXMI-Libs-Package"), dest.to_string(), true);
        if success {
            let finaldest = Path::new(&dest).join("xxmi.zip");
            let extract = extract_archive(finaldest.to_str().unwrap().to_string(), dest.to_string(), false);

            if extract {
                println!("xxmi extracted!");
            } else {
                println!("Failed to extract!");
            }
        } else {
            println!("failed to download xxmi");
        }
    }

    #[test]
    fn download_xxmi_packages_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/extras/xxmi/testing";
        let success = Extras::download_xxmi_packages(String::from("SilentNightSound/GIMI-Package"), String::from("SpectrumQT/SRMI-Package"), String::from("leotorrez/ZZMI-Package"), String::from("SpectrumQT/WWMI-Package"), String::from("leotorrez/HIMI-Package"), dest.to_string());
        if success {
            let d = Path::new(&dest);
            extract_archive(d.join("gimi.zip").to_str().unwrap().to_string(), d.join("gimi").to_str().unwrap().to_string(), false);
            extract_archive(d.join("srmi.zip").to_str().unwrap().to_string(), d.join("srmi").to_str().unwrap().to_string(), false);
            extract_archive(d.join("zzmi.zip").to_str().unwrap().to_string(), d.join("zzmi").to_str().unwrap().to_string(), false);
            extract_archive(d.join("wwmi.zip").to_str().unwrap().to_string(), d.join("wwmi").to_str().unwrap().to_string(), false);
            extract_archive(d.join("himi.zip").to_str().unwrap().to_string(), d.join("himi").to_str().unwrap().to_string(), false);
            println!("xxmi packages extracted!")
        } else {
            println!("Failed to download xxmi packages");
        }
    }

    /*#[test]
    fn download_runner_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/compatibility/runners/10.4-wine-vanilla";
        let url = "https://github.com/Kron4ek/Wine-Builds/releases/download/10.4/wine-10.4-amd64.tar.xz";

        let success = crate::compat::Compat::download_runner(url.to_string(), dest.to_string(), true);
        if success {
            println!("runner extracted!");
        } else {
            println!("failed to download runner");
        }
    }*/

    /*#[test]
    fn download_dxvk_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/compatibility/dxvk/2.6.0-vanilla";
        let url = "https://github.com/doitsujin/dxvk/releases/download/v2.6/dxvk-2.6.tar.gz";

        let success = crate::compat::Compat::download_dxvk(url.to_string(), dest.to_string(), true);
        if success {
            println!("dxvk extracted!");
        } else {
            println!("failed to download dxvk");
        }
    }*/

    #[test]
    fn download_fpsunlock_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/extras/fps_unlock/testing";

        let success = Extras::download_fps_unlock(String::from("mkrsym1/fpsunlock"), dest.to_string());
        if success {
            println!("fps unlock downloaded!")
        } else {
            println!("failed to download fpsunlock");
        }
    }

    #[test]
    fn download_jadeite_test() {
        let dest = "/home/tukan/.local/share/twintaillauncher/extras/jadeite/testing";

        let success = Extras::download_jadeite(String::from("mkrsym1/jadeite"), dest.to_string());
        if success {
            let finaldest = Path::new(dest).join("jadeite.zip");
            let extract = extract_archive(finaldest.to_str().unwrap().to_string(), dest.to_string(), false);

            if extract {
                println!("jadeite extracted!")
            } else {
                println!("Failed to extract!");
            }
        } else {
            println!("failed to download dxvk");
        }
    }

    // WARNING: Repair game test will take A REALLY LONG time!
    #[test]
    fn repair_game_test() {
        let res_list = String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/ScatteredFiles");
        let path = "/games/hoyo/hk4e_global/live";
        let rep = <Game as Hoyo>::repair_game(res_list, path.parse().unwrap(), false, |_, _| {});
        if rep { 
            println!("repair_game success!");
        } else {
            println!("repair_game failure!");
        }
    }

    // WARNING: Repair audio test will take A REALLY LONG time!
    #[test]
    fn repair_audio_test() {
        let res_list = String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/ScatteredFiles");
        let path = "/games/hoyo/hk4e_global/live";

        let rep = <Game as Hoyo>::repair_audio(res_list, VoiceLocale::English.to_folder().to_string(), path.parse().unwrap(), false, |_, _| {});
        if rep {
            println!("repair_audio success!");
        } else {
            println!("repair_audio failure!");
        }
    }

    #[test]
    fn download_fullgame_test() {
        let mut urls = Vec::<String>::new();
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.001"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.002"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.003"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.004"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.005"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.006"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.007"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/GenshinImpact_5.5.0.zip.008"));
        urls.push(String::from("https://autopatchhk.yuanshen.com/client_app/download/pc_zip/20250314110016_HcIQuDGRmsbByeAE/Audio_English(US)_5.5.0.zip"));

        let path = "/games/hoyo/hk4e_global/live/testing";
        let rep = <Game as Hoyo>::download(urls, path.parse().unwrap(), |current, total| {
            println!("current: {} | total: {}", current, total);
        });
        if rep {
            println!("full_game success!");
        } else {
            println!("full_game failure!");
        }
    }

    #[test]
    fn download_hdiff_test() {
        let url = "https://autopatchhk.yuanshen.com/client_app/update/hk4e_global/game_5.4.0_5.5.0_hdiff_IlvHovyEdpXnwiCH.zip";
        let path = "/games/hoyo/hk4e_global/live/testing";
        let rep = <Game as Hoyo>::patch(url.parse().unwrap(), path.parse().unwrap(), |current,total| {
            println!("current: {}, total: {}", prettify_bytes(current), prettify_bytes(total));
        });
        if rep {
            println!("diff_game success!");
        } else {
            println!("diff_game failure!");
        }
    }

    // WARNING: Repair game test will take A REALLY LONG time!
    #[tokio::test]
    async fn repair_game_kuro_test() {
        let manifest_pgr = "https://zspms-alicdn-gamestarter.kurogame.net/pcstarter/prod/game/G143/3.1.0.0/veNkEIbQIxPpTxlZvL1M9blBd3UmcUXh/resource.json";
        let manifest_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/resource/50004/2.3.1/indexFile.json";
        let chunkurl_pgr = "https://zspms-alicdn-gamestarter.kurogame.net/pcstarter/prod/game/G143/3.1.0.0/veNkEIbQIxPpTxlZvL1M9blBd3UmcUXh/zip";
        let chunkurl_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/zip";

        let path = "/games/kuro/wuwa_global/live/testing";
        let rep = <Game as Kuro>::repair_game(manifest_wuwa.to_string(), chunkurl_wuwa.to_string(), path.parse().unwrap(), false, |_, _| {}).await;
        if rep {
            println!("repair_game_kuro success!");
        } else {
            println!("repair_game_kuro failure!");
        }
    }

    #[tokio::test]
    async fn download_fullgame_kuro_test() {
        let manifest_pgr = "https://zspms-alicdn-gamestarter.kurogame.net/pcstarter/prod/game/G143/3.1.0.0/veNkEIbQIxPpTxlZvL1M9blBd3UmcUXh/resource.json";
        let manifest_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/resource/50004/2.3.1/indexFile.json";
        let chunkurl_pgr = "https://zspms-alicdn-gamestarter.kurogame.net/pcstarter/prod/game/G143/3.1.0.0/veNkEIbQIxPpTxlZvL1M9blBd3UmcUXh/zip";
        let chunkurl_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/zip";

        let path = "/games/kuro/wuwa_global/live/testing";
        let rep = <Game as Kuro>::download(manifest_wuwa.to_string(), chunkurl_wuwa.to_string(), path.parse().unwrap(), |current, total| {
            println!("current: {} | total: {}", current, total)
        }).await;
        if rep {
            println!("full_game_kuro success!");
        } else {
            println!("full_game_kuro failure!");
        }
    }

    #[tokio::test]
    async fn download_hdiff_kuro_test() {
        let manifest_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/resource/50004/2.3.1/1.0.0/indexFile.json";
        let chunkurl_wuwa = "https://hw-pcdownload-aws.aki-game.net/launcher/game/G153/2.3.1/axFplYInrILNAVwHsqPWvgirHzeKeBgS/zip";

        let path = "/games/kuro/wuwa_global/live/testing";
        let rep = <Game as Kuro>::patch(manifest_wuwa.to_string(), "2.3.0".to_string(), chunkurl_wuwa.to_string(), path.parse().unwrap(), false, |current,total| {
            println!("current: {}, total: {}", current, total);
        }).await;
        if rep {
            println!("diff_game_kuro success!");
        } else {
            println!("diff_game_kuro failure!");
        }
    }

    // Sophon
    #[tokio::test]
    async fn download_fullgame_hoyo_sophon_test() {
        let manifest = "https://autopatchhk.yuanshen.com/client_app/sophon/manifests/cxhpq4g4rgg0/q3h361jUEuu0/manifest_6194a90dbfdae455_f2f91f7cf5f869009f4816d8489f66ca";
        let chunkurl = "https://autopatchhk.yuanshen.com/client_app/sophon/chunks/cxhpq4g4rgg0/q3h361jUEuu0";
        let path = "/games/hoyo/hk4e_global/testing";

        let rep = <Game as Sophon>::download(manifest.to_string(), chunkurl.to_string(), path.parse().unwrap(), |current, total| {
            println!("current: {} | total: {}", current, total)
        }).await;
        if rep {
            println!("full_game_sophon success!");
        } else {
            println!("full_game_sophon failure!");
        }
    }

    #[tokio::test]
    async fn download_diff_hoyo_sophon_test() {
        let manifest = "https://autopatchhk.yuanshen.com/client_app/sophon/manifests/cxhpq4g4rgg0/DphJOTQP5dDn/manifest_68690fa573e2b399_07f4d65a4f4d42157f863d7d177efaa4";
        let chunkurl = "https://autopatchhk.yuanshen.com/client_app/sophon/diffs/cxhpq4g4rgg0/DphJOTQP5dDn/10016";
        let path = "/games/hoyo/hk4e_global/live/testing";

        let rep = <Game as Sophon>::patch(manifest.to_string(), "5.6.0".to_string(), chunkurl.to_string(), path.parse().unwrap(), "".to_string(), true, false, |current, total| {
            println!("current: {} | total: {}", current, total)
        }).await;
        if rep {
            println!("diff_game_sophon success!");
        } else {
            println!("diff_game_sophon failure!");
        }
    }

    #[tokio::test]
    async fn repair_hoyo_sophon_test() {
        let manifest = "https://autopatchhk.yuanshen.com/client_app/sophon/manifests/cxhpq4g4rgg0/3j90WyoOpfjt/manifest_b39180b4ad2cabd3_5bd9ec245492bc76dc13c0204b43cce8";
        let chunkurl = "https://autopatchhk.yuanshen.com/client_app/sophon/chunks/cxhpq4g4rgg0/3j90WyoOpfjt";
        let path = "/games/hoyo/hk4e_global/live/testing";

        let rep = <Game as Sophon>::repair_game(manifest.to_string(), chunkurl.to_string(), path.parse().unwrap(), false,|current, total| {
            println!("current: {} | total: {}", current, total)
        }).await;
        if rep {
            println!("repair_game_sophon success!");
        } else {
            println!("repair_game_sophon failure!");
        }
    }

    #[tokio::test]
    async fn download_preload_hoyo_sophon_test() {
        let manifest = "https://autopatchhk.yuanshen.com/client_app/sophon/manifests/cxhpq4g4rgg0/DphJOTQP5dDn/manifest_68690fa573e2b399_07f4d65a4f4d42157f863d7d177efaa4";
        let chunkurl = "https://autopatchhk.yuanshen.com/client_app/sophon/diffs/cxhpq4g4rgg0/DphJOTQP5dDn/10016";
        let path = "/games/hoyo/hk4e_global/live/testing";

        let rep = <Game as Sophon>::preload(manifest.to_string(), "5.6.0".to_string(), chunkurl.to_string(), path.parse().unwrap(), |current, total| {
            println!("current: {} | total: {}", current, total)
        }).await;
        if rep {
            println!("preload_game_sophon success!");
        } else {
            println!("preload_game_sophon failure!");
        }
    }
}
