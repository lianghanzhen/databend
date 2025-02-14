// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use common_datavalues::prelude::*;
use common_exception::Result;
use common_functions::scalars::UpperFunction;

use crate::scalars::scalar_function2_test::test_scalar_functions;
use crate::scalars::scalar_function2_test::ScalarFunctionTest;

#[test]
fn test_upper_function() -> Result<()> {
    let tests = vec![
        ScalarFunctionTest {
            name: "upper-abc-passed",
            columns: vec![Series::from_data(vec!["Abc"])],
            expect: Series::from_data(vec!["ABC"]),
            error: "",
        },
        ScalarFunctionTest {
            name: "upper-utf8-passed",
            columns: vec![Series::from_data(vec!["Dobrý den"])],
            expect: Series::from_data(vec!["DOBRÝ DEN"]),
            error: "",
        },
        ScalarFunctionTest {
            name: "ucase-utf8-passed",
            columns: vec![Series::from_data(vec!["Dobrý den"])],
            expect: Series::from_data(vec!["DOBRÝ DEN"]),
            error: "",
        },
    ];

    test_scalar_functions(UpperFunction::try_create("upper")?, &tests, true)
}

#[test]
fn test_upper_nullable() -> Result<()> {
    let tests = vec![ScalarFunctionTest {
        name: "ucase-null-passed",
        columns: vec![Series::from_data(vec![Option::<Vec<u8>>::None])],
        expect: Series::from_data(vec![Option::<Vec<u8>>::None]),
        error: "",
    }];

    test_scalar_functions(UpperFunction::try_create("ucase")?, &tests, true)
}
