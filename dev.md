# Dev
This file includes some development related notes.

### Debug Tips
1. Remember to set your network before sending requests to lotus: `set_current_network(Network::Testnet)`. Or else your address might not be parsed.
2. Return results from lotus are usually base64 encoded to string (based on test results). Might be helpful when converting string to struct.
