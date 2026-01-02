## [1.4.1](https://github.com/OmentaElvis/pesa-playground/compare/v1.4.0...v1.4.1) (2026-01-01)


### Bug Fixes

* **core:** fix db nuke logic that failed due to FK constraints ([5e4872a](https://github.com/OmentaElvis/pesa-playground/commit/5e4872aecf2a5461827b462e9d927695672ffee1))
* **users:** users were generating with absurd high values due to currency unit mismatch ([8bd31c0](https://github.com/OmentaElvis/pesa-playground/commit/8bd31c0c008f48377d44384415f33e2d590b9800))

# [1.4.0](https://github.com/OmentaElvis/pesa-playground/compare/v1.3.1...v1.4.0) (2026-01-01)


### Bug Fixes

* **b2c:** correctly dispatch transaction events on disbursment complete ([14e4242](https://github.com/OmentaElvis/pesa-playground/commit/14e4242c9a9f083a2903b54a34b73d3bbc8a5a54))
* **b2c:** decode security credential from base64 before decrypting ([81d15a7](https://github.com/OmentaElvis/pesa-playground/commit/81d15a716f04e7634a97cbc975a467e460e2d21e))


### Features

* **api:** added balance inquiry api ([86ff50d](https://github.com/OmentaElvis/pesa-playground/commit/86ff50d3066c7e6140d3547dff2c173d1b124231))
* **api:** include internal error message in api error headers ([07a61d0](https://github.com/OmentaElvis/pesa-playground/commit/07a61d0d2fa169fe2f37c586b76b88ef267079ca))
* **b2c:** added self test for b2c disbursment ([24f9882](https://github.com/OmentaElvis/pesa-playground/commit/24f98828ae4ec5eeb16dd1ea8d8f45c8c59c05ad))

## [1.3.1](https://github.com/OmentaElvis/pesa-playground/compare/v1.3.0...v1.3.1) (2026-01-01)


### Bug Fixes

* **ci:** tauri windows build upload the correct artifact version ([baa7a9b](https://github.com/OmentaElvis/pesa-playground/commit/baa7a9b6dd4fc2ad01c0c9cef9343d7bfafc8b89))

# [1.3.0](https://github.com/OmentaElvis/pesa-playground/compare/v1.2.0...v1.3.0) (2026-01-01)


### Bug Fixes

* **core:** fail to insert callback log causing failed api callback delivery ([3ba512a](https://github.com/OmentaElvis/pesa-playground/commit/3ba512a6282cecc7705527dc072bdc37f03cc52e))
* **core:** unable to transfer funds from system account ([f355d48](https://github.com/OmentaElvis/pesa-playground/commit/f355d483f42bd1decf252226af45e5e99959acd1))
* **web:** include fields for storing custom keymaps ([9c6cf5d](https://github.com/OmentaElvis/pesa-playground/commit/9c6cf5d07a2aef4fa34ed4ff6df51f8b81ad4024))
* **web:** wrong fetch of users list type ([f55c9ce](https://github.com/OmentaElvis/pesa-playground/commit/f55c9cee5eb0af2a6a5b9f01733c53246ba6115f))


### Features

* add ability to transfer funds to utility account from mmf ([6a0e2d2](https://github.com/OmentaElvis/pesa-playground/commit/6a0e2d21116b4d0373d509e7b309d6167148c38d))
* added [project url]/debug/config ([a2a9bea](https://github.com/OmentaElvis/pesa-playground/commit/a2a9bea0abf4cc5b69bd25105e6df279f403799b))
* added [project url]/debug/users ([f7c1413](https://github.com/OmentaElvis/pesa-playground/commit/f7c1413da4701aa87c57cea4083eec3dae13a4d5))
* added self diagnostics for core api ([fd571f7](https://github.com/OmentaElvis/pesa-playground/commit/fd571f76aaccdb81bf0d993b13858038d44ee8a4))
* **core:** added db migration strategy ([d8a84f2](https://github.com/OmentaElvis/pesa-playground/commit/d8a84f252e9778221bf0b563fcd4f47d001c8d1b))
* **core:** generic async request handler for all apis ([e4e8bf4](https://github.com/OmentaElvis/pesa-playground/commit/e4e8bf4a5c1841d52e2569d9e1b5d6129d7ba7df))
* **users:** include imsi and registered date to users table ([8d2763f](https://github.com/OmentaElvis/pesa-playground/commit/8d2763fbc7b76746cd7c2e7e157c6fef0b6a6e9a))
* **web:** auto refresh transaction list on new transaction event ([a671129](https://github.com/OmentaElvis/pesa-playground/commit/a6711296af61c065652739d6404b07e4f08d2964))

# [1.2.0](https://github.com/OmentaElvis/pesa-playground/compare/v1.1.1...v1.2.0) (2025-12-25)


### Bug Fixes

* **ci:** ensure Tauri builds use correct release version ([931ab24](https://github.com/OmentaElvis/pesa-playground/commit/931ab24e96705f553b878187c8d1c738047265ab))
* **web:** fix race condition in displaying keymaps on settings page ([fce8b29](https://github.com/OmentaElvis/pesa-playground/commit/fce8b29be8d9a331c424aab3531cb75fbce62380))
* **web:** resolve sandbox status initialization race condition ([f5ca628](https://github.com/OmentaElvis/pesa-playground/commit/f5ca62807d3e3e783b58aabdadea7f53c4758c5e))
* **web:** synchronize sandbox UI state with the central store ([e8946a9](https://github.com/OmentaElvis/pesa-playground/commit/e8946a911a6351a585e7e507b2e6ccde4653ee54))


### Features

* persist custom keymaps to backend settings ([d100d6f](https://github.com/OmentaElvis/pesa-playground/commit/d100d6ff19c9e77d10f65caef3ec27f170edc27b))


### Performance Improvements

* use single use animations and avoid continious resource heavy animations ([f61c836](https://github.com/OmentaElvis/pesa-playground/commit/f61c8365ab10b32a10a6c61355c02330c212bad5))

## [1.1.1](https://github.com/OmentaElvis/pesa-playground/compare/v1.1.0...v1.1.1) (2025-12-25)


### Bug Fixes

* **ci:** tauri.conf.json fix NEW_VERSION command not found ([0064860](https://github.com/OmentaElvis/pesa-playground/commit/0064860a6d5cf1060d33b84f659218b0e98d8c38))

# [1.1.0](https://github.com/OmentaElvis/pesa-playground/compare/v1.0.0...v1.1.0) (2025-12-25)


### Bug Fixes

* **c2b:** fixed c2b process logic to check for empty strings instead of just null ([ca90bd2](https://github.com/OmentaElvis/pesa-playground/commit/ca90bd2353467a4650e98ea30f1183bd89fd7099))
* **ui:** fix chat bubble max width ([6ae8c88](https://github.com/OmentaElvis/pesa-playground/commit/6ae8c887951cfacdc0ab63f9e4aadd5095fe10c2))
* **ui:** lock user list to 260px ([8270e97](https://github.com/OmentaElvis/pesa-playground/commit/8270e97e51ccc3ac0a8464b68defc9dc2221c658))
* **ui:** remove user avatar border ([abb0a57](https://github.com/OmentaElvis/pesa-playground/commit/abb0a570b3383c325acc84ac18d3ca175fc964ee))


### Features

* added api endpoint for copying in project page ([797aee0](https://github.com/OmentaElvis/pesa-playground/commit/797aee04cf13a9da44c009aa324e0f304673d2d8))
* added business operators ([2cc9817](https://github.com/OmentaElvis/pesa-playground/commit/2cc9817291f3376181188400fb37de317d2878be))
* added generate b2c security credential page ([7b461cd](https://github.com/OmentaElvis/pesa-playground/commit/7b461cd0ee20d4b9a88ed052696899285dfea94b))
* added persistent backend settings ([f2b64b1](https://github.com/OmentaElvis/pesa-playground/commit/f2b64b138337d3e28d143f387e8e9786ec383f47))
* **ui:** added toggle for message and table view in user transactions ([7ff4da8](https://github.com/OmentaElvis/pesa-playground/commit/7ff4da8f6881c269476becec6aec6f170cb1fa18))
* **ui:** connect lipa na mpesa to stk menu ([dba2dc7](https://github.com/OmentaElvis/pesa-playground/commit/dba2dc7411b1e5c783b8e7e2f71d7707611e169e))
* **ui:** users ui tweaks ([ef25b6a](https://github.com/OmentaElvis/pesa-playground/commit/ef25b6a4741ff6eb12d2bdce387e9ef2786e743b))


### Performance Improvements

* use async events to communicate sandbox status instead of active poll ([d74542d](https://github.com/OmentaElvis/pesa-playground/commit/d74542dc8134599778ad68bb2de1b5a83beafcc9))

# 1.0.0 (2025-12-22)


### Bug Fixes

* **ci:** add contents read permission to commitlint action ([dc1d3ed](https://github.com/OmentaElvis/pesa-playground/commit/dc1d3ede203eb501d49a6250ea6751345bb7d4f7))
* **ci:** add execute permission to .github/workflows/prepare-release.sh ([dbca8f4](https://github.com/OmentaElvis/pesa-playground/commit/dbca8f4bf16845b481efe570aca6b3c7b1807b2c))
* **ci:** change the jsonpath for toml to correct syntax ([a7a39a3](https://github.com/OmentaElvis/pesa-playground/commit/a7a39a3d127afea943cbfaa6e4b2cdb425c539a8))
* **ci:** correct Docker image tagging with release version ([df519ba](https://github.com/OmentaElvis/pesa-playground/commit/df519ba10108b518d92a5d0e21dee93d0a9d1485))
* **ci:** correctly parse semantic-release output to unblock builds and reset CHANGELOG.md ([38bdb2c](https://github.com/OmentaElvis/pesa-playground/commit/38bdb2c1274756f0f74310ad8f380e27df511477))
* **ci:** correctly parse semantic-release output to unblock builds and reset CHANGELOG.md ([f89c1b6](https://github.com/OmentaElvis/pesa-playground/commit/f89c1b60ed0f683c3e57901374c034ff0aef8eee))
* **ci:** correctly setup tauri requirements for cargo check ([ba30985](https://github.com/OmentaElvis/pesa-playground/commit/ba309854cd6ae55fb098520d1de84c76139f3e5b))
* **ci:** fixed windows and linux tauri builds ([50466d5](https://github.com/OmentaElvis/pesa-playground/commit/50466d533b9a96b76b24909ba160cd88f0cb28d5))
* **ci:** fixed windows tauri unknown bundles nsis ([7da4ade](https://github.com/OmentaElvis/pesa-playground/commit/7da4ade72102fcaf68b59657b583787e90f0a3e6))
* **ci:** include missing rpm bundle in tauri ([105938e](https://github.com/OmentaElvis/pesa-playground/commit/105938ea30a365346de00a3e67f99ed1aaaa971a))
* **ci:** module warning MODULE_TYPELESS_PACKAGE_JSON ([aa2b098](https://github.com/OmentaElvis/pesa-playground/commit/aa2b098dd93a0c11bc4e43407a5b29d0907a0883))
* **ci:** remove release-type simple from release-please workflow ([b2bde49](https://github.com/OmentaElvis/pesa-playground/commit/b2bde49854d4279e187cb0d8385042304277f7f0))
* **ci:** restore @semantic-release/github ([1aaf1f3](https://github.com/OmentaElvis/pesa-playground/commit/1aaf1f39d9d441078c71dbd7ece305b51035db74))
* **ci:** set release-type to simple for PR-based releases ([5c2e8b6](https://github.com/OmentaElvis/pesa-playground/commit/5c2e8b6dc39d6acd4e15b73553a8c842d96ef09f))
* **clippy:** fixed clippy warnings ([f6c9e24](https://github.com/OmentaElvis/pesa-playground/commit/f6c9e241d3922ee4e657b59ab3a9085057fa507b))
* **docker:** correctly handle pnpm workspace setup ([eebcc91](https://github.com/OmentaElvis/pesa-playground/commit/eebcc91ec6bef564a2ca6e3ddd2e0548bfe32dbf))
* duplicate key `edition` in table `package` in crates/pesa-macros ([e59ea22](https://github.com/OmentaElvis/pesa-playground/commit/e59ea2226f8fb0ecd4ce73e8372688599c344860))
* Removed duplicate users entry on sidebar ([734d28b](https://github.com/OmentaElvis/pesa-playground/commit/734d28bac0bf9d6b49001ed251d2633a14168848))
* **rust:** add unsafe block to end::set_var for 2024 edition ([b99caa0](https://github.com/OmentaElvis/pesa-playground/commit/b99caa054b14c6f3373508d1045fb851bfd95b9c))
* **tauri:** allow minimization of app window ([05faaa9](https://github.com/OmentaElvis/pesa-playground/commit/05faaa9d3262608577e90512c773314240bf9358))
* **tauri:** export missing get/create_account commands ([703f273](https://github.com/OmentaElvis/pesa-playground/commit/703f273a6ebbfe5df9d84207f4cb4834c15bd669))
* **users:** fixed wrong dropdown menu text color ([fc97f2f](https://github.com/OmentaElvis/pesa-playground/commit/fc97f2f17f83253b642d5ce23095ffb62ae31fa9))
* **users:** ui now reacts to generated users ([6c4b58f](https://github.com/OmentaElvis/pesa-playground/commit/6c4b58fa07eae5e71891b42d28e75054af9975ad))


### Features

* added configurable logo component ([8434e1c](https://github.com/OmentaElvis/pesa-playground/commit/8434e1ca339a6d0a591b7883d7dc995127592bd7))
* added customizable keymap system. Added shortcuts to start and stop sandboxes ([2e1e7a0](https://github.com/OmentaElvis/pesa-playground/commit/2e1e7a0642ef1bd566716c23be0ae7d6a2e23d62))
* added keymap system ([2649e2a](https://github.com/OmentaElvis/pesa-playground/commit/2649e2a90733b220c8919f8e2a7fa7315a524ba5))
* added projects welcome page ([be76d44](https://github.com/OmentaElvis/pesa-playground/commit/be76d440b55350563f8edec05de8b9fca8e6d504))
* **app-reset:** implement app data purge and reset functionality ([f081db8](https://github.com/OmentaElvis/pesa-playground/commit/f081db8e6296e7de9388a1ba29cbbe2862c77a7e))
* **ci:** configure release-please with build jobs ([82f52ec](https://github.com/OmentaElvis/pesa-playground/commit/82f52ec07501c15ca812ac997891be8f0dc66b1b))
* **ci:** implement unified release with artifact builds ([5b2f370](https://github.com/OmentaElvis/pesa-playground/commit/5b2f3705d7b50944f947455b58ba67ca37221443))
* improved pesa playground logo ([bae58ab](https://github.com/OmentaElvis/pesa-playground/commit/bae58ab3c9826d926333fce0214143bba46f799a))
