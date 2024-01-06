# Ipic Vision

An open source image API which can return images according to the IP.

## Usage

Download the bin file in release or build manually.

To config the API, you need to create `config.json`.

A config example:

```json
{
    "listen_addr": "127.0.0.1:9090",
    "traffic_matchers": [
      {
        "role": {
          "ipv4_exact": "192.168.1.1"
        },
        "image": {
          "random_list": [
            {
              "Url": "https://example.com/1.jpg"
            },
            {
              "Url": "https://example.com/2.jpg"
            }
          ]
        }
      },
      {
        "role": "default",
        "image": {
          "one": {
            "Url": "https://example.com/1.jpg"
          }
        }
      }
    ]
}

```

If you need to match IP with country, you need to have an ip-info token.

```json
{
    "ip_info_token": "your ipinfo token",
    "ip_info_enable": true,
    "listen_addr": "127.0.0.1:9090"
}
```

## Config Details

To run the API, you must config a `listen_addr` and `traffic_matchers`.

To match all traffic, you can use `role: default`.

The minimal config is:

```json
{
    "listen_addr": "127.0.0.1:9090",
    "ip_info_enable": false,
    "traffic_matchers": [
      {
        "role": "default",
        "image": {
          "one": {
            "Url": "https://example.com/1.jpg"
          }
        }
      }
    ]
}
```

The `tarffic_matchers` will be matched one by one from top to bottom. If no match, the API will return 404.

The `"role": "default"` will match all traffic.

## `traffic_matchers` Config

The `traffic_matchers` is an array of `traffic_matcher`.

The `traffic_matcher` is an object with `role` and `image`.

## `role` Config

The `role` config can be a string or an object.

These options are **only supported for ipv6**:
+ `ipv6_default`: match all ipv6 traffic.

These options are **only supported for ipv4**:

+ `ipv4_exact`: match the exact IP.
+ `ipv4_maske`: match the IP with mask.
+ `ipv4_cidr`: match the IP with CIDR.
+ `ipv4_default`: match all ipv4 traffic.

These options are **supported for both ipv4 and ipv6**:
+ `region`: match the region of the IP.
+ `default`: match all traffic.

Expect `ipv4_masked`, `ipv4_default`, `ipv6_default` and `default`, all config should like this:

```json
{
  "role": {
    "ipv4_exact": "192.168.1.1"
  }
}
```

Especially, the `ipv4_default` and `ipv4_masked` config should like this:

```json
{
  "role": "ipv4_default"
}
```

```json
{
  "role": {
    "ipv4_masked": {
        "ip": "192.168.1.1",
        "mask": "255.255.0.0"
    }
  }
}
```

## `image` Config

The `image` config is an object with `one` or `random_list`.

+ The `one` config will return the same image every time.
+ The `random_list` config will return a random image from the list.

Each image is an object whose key is `Url` or `Path`:

Example1: return same image (on disk) every time.

```json
{
    "image": {
        "one": {
          "Path": "/path/to/image.jpg"
        }
    }
}
```

Example2: return random image (on Internet as URL)

```json
{
    "image": {
        "random_list": [
          {
            "Url": "https://example.com/1.jpg"
          },
          {
            "Url": "https://example.com/2.jpg"
          }
        ]
    }
}
```