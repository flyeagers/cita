// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use self::bincode::internal::serialize_into;
use self::bincode::Infinite;
use super::factory::Contract;

use bincode;
use cita_types::{H256, U256};
use std::io::Write;

use crate::cita_executive::VmExecParams;
use crate::context::Context;
use crate::contracts::tools::method as method_tools;
use crate::storage::{Array, Map, Scalar};
use crate::types::errors::NativeError;

use byteorder::BigEndian;
use cita_vm::evm::DataProvider;
use cita_vm::evm::InterpreterResult;

#[derive(Clone)]
pub struct SimpleStorage {
    uint_value: Scalar,
    string_value: Scalar,
    array_value: Array,
    map_value: Map,
    output: Vec<u8>,
}

impl Contract for SimpleStorage {
    fn exec(
        &mut self,
        params: &VmExecParams,
        _context: &Context,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        method_tools::extract_to_u32(&params.data[..]).and_then(|signature| match signature {
            0 => self.init(params, data_provider),
            0xaa91543e => self.uint_set(params, data_provider),
            0x832b4580 => self.uint_get(params, data_provider),
            0xc9615770 => self.string_set(params, data_provider),
            0xe3135d14 => self.string_get(params, data_provider),
            0x118b229c => self.array_set(params, data_provider),
            0x180a4bbf => self.array_get(params, data_provider),
            0xaaf27175 => self.map_set(params, data_provider),
            0xc567dff6 => self.map_get(params, data_provider),
            _ => Err(NativeError::Internal("out of gas".to_string())),
        })
    }
    fn create(&self) -> Box<dyn Contract> {
        Box::new(SimpleStorage::default())
    }
}

impl Default for SimpleStorage {
    fn default() -> Self {
        SimpleStorage {
            output: Vec::new(),
            uint_value: Scalar::new(H256::from(0)),
            string_value: Scalar::new(H256::from(1)),
            array_value: Array::new(H256::from(2)),
            map_value: Map::new(H256::from(3)),
        }
    }
}

impl SimpleStorage {
    fn init(
        &mut self,
        _params: &VmExecParams,
        _ext: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        Ok(InterpreterResult::Normal(vec![], 100, vec![]))
    }

    // 1) uint
    fn uint_set(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let value = U256::from(params.data.get(4..36).expect("no enough data"));
        self.uint_value
            .set(data_provider, &params.code_address, value)?;
        Ok(InterpreterResult::Normal(vec![], 100, vec![]))
    }

    fn uint_get(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        self.output.resize(32, 0);
        self.uint_value
            .get(data_provider, &params.code_address)?
            .to_big_endian(self.output.as_mut_slice());
        Ok(InterpreterResult::Normal(self.output.clone(), 100, vec![]))
    }

    // 2) string
    fn string_set(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let data = params.data.to_owned();
        let index = U256::from(data.get(4..36).expect("no enough data")).low_u64() as usize + 4;
        let length =
            U256::from(data.get(index..(index + 32)).expect("no enough data")).low_u64() as usize;
        let index = index + 32;
        let value = String::from_utf8(Vec::from(
            data.get(index..index + length).expect("no enough data"),
        ))
        .unwrap();

        self.string_value
            .set_bytes(data_provider, &params.code_address, &value)?;
        Ok(InterpreterResult::Normal(vec![], 100, vec![]))
    }

    fn string_get(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        self.output.resize(0, 0);
        let str = self
            .string_value
            .get_bytes::<String>(data_provider, &params.code_address)?;
        for i in U256::from(32).0.iter().rev() {
            serialize_into::<_, _, _, BigEndian>(&mut self.output, &i, Infinite)
                .expect("failed to serialize u64");
        }
        for i in U256::from(str.len()).0.iter().rev() {
            serialize_into::<_, _, _, BigEndian>(&mut self.output, &i, Infinite)
                .expect("failed to serialize u64");
        }

        for i in str.bytes() {
            serialize_into::<_, _, _, BigEndian>(&mut self.output, &i, Infinite)
                .expect("failed to serialize ");
        }
        self.output
            .write(&vec![0u8; 32 - str.len() % 32])
            .expect("failed to write [u8]");
        Ok(InterpreterResult::Normal(self.output.clone(), 100, vec![]))
    }

    // 3) array
    fn array_set(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let data = params.data.to_owned();
        let mut pilot = 4;
        let index = U256::from(data.get(pilot..pilot + 32).expect("no enough data")).low_u64();
        pilot += 32;
        let value = U256::from(data.get(pilot..pilot + 32).expect("no enough data"));
        self.array_value
            .set(data_provider, &params.code_address, index, &value)?;
        Ok(InterpreterResult::Normal(vec![], 100, vec![]))
    }

    fn array_get(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let data = params.data.to_owned();
        let index = U256::from(data.get(4..4 + 32).expect("no enough data")).low_u64();
        for i in self
            .array_value
            .get(data_provider, &params.code_address, index)?
            .0
            .iter()
            .rev()
        {
            serialize_into::<_, _, _, BigEndian>(&mut self.output, &i, Infinite)
                .expect("failed to serialize u64");
        }
        Ok(InterpreterResult::Normal(self.output.clone(), 100, vec![]))
    }

    // 4) map
    fn map_set(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let data = params.data.to_owned();
        let mut pilot = 4;
        let key = U256::from(data.get(pilot..pilot + 32).expect("no enough data"));
        pilot += 32;
        let value = U256::from(data.get(pilot..pilot + 32).expect("no enough data"));
        self.map_value
            .set(data_provider, &params.code_address, &key, value)?;
        Ok(InterpreterResult::Normal(vec![], 100, vec![]))
    }

    fn map_get(
        &mut self,
        params: &VmExecParams,
        data_provider: &mut dyn DataProvider,
    ) -> Result<InterpreterResult, NativeError> {
        let data = params.data.to_owned();
        let key = U256::from(data.get(4..4 + 32).expect("no enough data"));
        for i in self
            .map_value
            .get(data_provider, &params.code_address, &key)?
            .0
            .iter()
            .rev()
        {
            serialize_into::<_, _, _, BigEndian>(&mut self.output, &i, Infinite)
                .expect("failed to serialize u64");
        }
        Ok(InterpreterResult::Normal(self.output.clone(), 100, vec![]))
    }
}

#[test]
fn test_native_contract() {
    use super::factory::Factory;
    use crate::cita_executive::VmExecParams;
    use crate::context::Context;
    use crate::tests::exemock::DataProviderMock;
    use crate::types::reserved_addresses;
    use cita_types::Address;
    use std::str::FromStr;

    let factory = Factory::default();
    let context = Context::default();
    let native_addr = Address::from_str(reserved_addresses::NATIVE_SIMPLE_STORAGE).unwrap();
    let mut data_provider = DataProviderMock::default();
    let value = U256::from(0x1234);
    {
        let mut params = VmExecParams::default();
        params.code_address = Address::from("0x4b5ae4567ad5d9fb92bc9afd6a657e6fa13a2523");
        let mut input = Vec::new();
        let index = 0xaa91543eu32;
        serialize_into::<_, _, _, BigEndian>(&mut input, &index, Infinite)
            .expect("failed to serialize u32");
        for i in value.0.iter().rev() {
            serialize_into::<_, _, _, BigEndian>(&mut input, &i, Infinite)
                .expect("failed to serialize u64");
        }
        params.data = input;
        let mut contract = factory.new_contract(native_addr).unwrap();
        let _output = contract
            .exec(&params, &context, &mut data_provider)
            .expect("Set value failed.");
    }
    {
        let mut input = Vec::new();
        let mut params = VmExecParams::default();
        params.code_address = Address::from("0x4b5ae4567ad5d9fb92bc9afd6a657e6fa13a2523");
        let index = 0x832b4580u32;
        serialize_into::<_, _, _, BigEndian>(&mut input, &index, Infinite)
            .expect("failed to serialize u32");
        params.data = input;

        let mut contract = factory.new_contract(native_addr).unwrap();
        match contract.exec(&params, &context, &mut data_provider) {
            Ok(InterpreterResult::Normal(return_data, _quota_left, _logs)) => {
                let real = U256::from(&*return_data);
                assert!(real == value);
            }
            _ => assert!(false, "no output data"),
        };
    }
}
