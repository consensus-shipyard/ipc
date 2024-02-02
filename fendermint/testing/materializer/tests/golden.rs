// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod json {
    use fendermint_testing::golden_json;
    use fendermint_testing_materializer::manifest::Manifest;
    use quickcheck::Arbitrary;
    golden_json! { "manifest/json", manifest, Manifest::arbitrary }
}

mod yaml {
    use fendermint_testing::golden_yaml;
    use fendermint_testing_materializer::manifest::Manifest;
    use quickcheck::Arbitrary;
    golden_yaml! { "manifest/yaml", manifest, Manifest::arbitrary }
}
