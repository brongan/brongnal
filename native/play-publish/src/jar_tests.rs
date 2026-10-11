use super::*;

fn bundle() -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    writer.add_directory("base/", options).unwrap();
    writer.start_file("AndroidManifest.xml", options).unwrap();
    writer.write_all(b"<manifest/>").unwrap();
    writer
        .start_file(format!("base/res/{}.png", "x".repeat(80)), options)
        .unwrap();
    writer.write_all(b"png").unwrap();
    writer.finish().unwrap().into_inner()
}

#[test]
fn signature_files_digest_each_entry_and_section() {
    let files = signature_files(&bundle()).unwrap();
    assert_eq!(
        String::from_utf8(files.manifest).unwrap(),
        concat!(
            "Manifest-Version: 1.0\r\n",
            "\r\n",
            "Name: AndroidManifest.xml\r\n",
            "SHA-256-Digest: GK9kZ8USIAcMayUVthOT5QfWfhyxpDaKHHWLejJw+Eg=\r\n",
            "\r\n",
            "Name: base/res/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\r\n",
            " xxxxxxxxxxxxxxxxxxxxxxx.png\r\n",
            "SHA-256-Digest: j4y7fc9G4Lx9UyZXSabBfRFgk6a6leRCdkBgx2/UqGw=\r\n",
            "\r\n",
        )
    );
    assert_eq!(
        String::from_utf8(files.sf).unwrap(),
        concat!(
            "Signature-Version: 1.0\r\n",
            "SHA-256-Digest-Manifest: 17Qjq6qIOAjrd+UnqtOpHKzmYlTMHfU6AlWO99E9P6k=\r\n",
            "\r\n",
            "Name: AndroidManifest.xml\r\n",
            "SHA-256-Digest: yDw3XjYy+ZfCEltUlVH4m9GULmLWiYc5Z/kYiEjLHm0=\r\n",
            "\r\n",
            "Name: base/res/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\r\n",
            " xxxxxxxxxxxxxxxxxxxxxxx.png\r\n",
            "SHA-256-Digest: Rz1DX7IxEAOpGlHLX50z4CM+LS3wiYIKAyFPgw5S7ko=\r\n",
            "\r\n",
        )
    );
}
