{ pkgs }: {
	deps = [
    pkgs.git
    pkgs.coreutils
    pkgs.rustup
		pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
		pkgs.cargo-edit
    pkgs.rust-analyzer
	];
}