# Todos
* authentication / authorization
* implement `expose-internal-error` feature
  * most of the errors should be InternalError
* implement graphql client
  * query
  * mutation
  * subscription
* implements tests
* implement schema differentiator
  * param for specifying wether only breaking changes should be shown; a breaking change is when
    * types that were optional, became required
    * a field is removed
    * an input type argument is removed
    * an input type argument is added without default value
    * a type implements an interface
    * a type no longer implements an interface
    * etc.

# Example queries
## Query
```
{
  me {
    id
    displayName
    characters {
      race
      nickname
      name
    }
  }
}
```

## Mutation
```
mutation {
  createCharacter(
    userId: "e30ba9c8-03bf-4ae8-af35-e8366a8fe160"
    characterDefinition: { race: ANDROID, nickname: "Telinia" }
  ) {
    nickname
  }
}
```
