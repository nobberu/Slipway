// Copyright (C) 2026 nobberu <itsnobberu@gmail.com>
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    slint_build::compile("ui/app.slint").unwrap();

    #[cfg(target_os = "windows")]
        {
            let mut res = winres::WindowsResource::new();
            res.set("ProductName", "Slipway");
            res.set("FileDescription", "Slipway");
            res.compile().unwrap();
        }
}
