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
