# awa

this is awa programming language.

awa is staticly typed, interpreted, and has on the fly code hotswap features

## why awa?

- **trustworthiness:** best-in-class code hotswap features allow for quick and
  secure changes in parts of the source code during the execution
- **performance:** blazingly fast startup and execution paired with
  negligible memory usage
- **efficiency:** only needed features and very readable syntax make it ideal for
  anyone to become productive in under an hour

## code example

```awa
func say_hello(name string) {
    if (name == "") {
        name = "世界"
    }

    println("Hello, " <> name <> "!")
}

func main() {
    var name string = "awa"
    say_hello(name)
}
```

