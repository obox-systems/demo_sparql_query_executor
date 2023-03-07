use serde_json::Value;

async fn pull_data_with_sparql( query : &str ) -> Value 
{

    let api = mediawiki::api::Api::new( "https://www.wikidata.org/w/api.php" )
        .await
        .unwrap(); // Will determine the SPARQL API URL via site info data
    let res = api.sparql_query( query ).await.unwrap();
    res

}

#[ tokio::main ]
async fn main() {
    let query = r#"
    SELECT ?countryLabel (COUNT(?city) as ?count) WHERE {
        ?country wdt:P31 wd:Q3624078.
        ?city wdt:P31/wdt:P279* wd:Q515.
        ?city wdt:P1082 ?population.
        ?city wdt:P131 ?country.
        FILTER(?population >= 500000).
        SERVICE wikibase:label {
          bd:serviceParam wikibase:language "[AUTO_LANGUAGE],en".
        }
      }
      GROUP BY ?country ?countryLabel
      ORDER BY DESC(?count)
    "#;

    let res = pull_data_with_sparql( query ).await;
    println!( "{}", serde_json::to_string_pretty( &res ).unwrap() );
}

#[ cfg( test ) ]
mod test 
{

    use super::*;

    #[ tokio::test ]
    async fn select_query() 
    {

        let query = r#"
        SELECT ?tajmahal ?tajmahalLabel ?tajmahalDescription ?image WHERE {
          wd:Q178 wdt:P31 wd:Q39446.
          OPTIONAL { wd:Q178 wdt:P18 ?image. }
          SERVICE wikibase:label { bd:serviceParam wikibase:language "[AUTO_LANGUAGE],en". }
        }
        "#;
        let result = pull_data_with_sparql( query ).await;
        assert!( result.to_string().contains( "tajmahal" ) );

    }

    #[ tokio::test ]
    async fn construct_query() 
    {

        let query = r#"
        CONSTRUCT WHERE { ?s ?p ?o } LIMIT 1
        "#;
        let result = pull_data_with_sparql( query ).await;
        assert!( result.to_string().contains( "uri" ) );

    }

    #[ tokio::test ]
    async fn describe_query() 
    {

        let query = r#"
        DESCRIBE <https://en.wikipedia.org/wiki/Rust_(programming_language)>
        "#;
        let result = pull_data_with_sparql( query ).await;
        assert!( result.to_string().contains( "Rust" ) );

    }

    #[ tokio::test ]
    async fn ask_query() 
    {

        let query = r#"
        ASK WHERE {
            <http://dbpedia.org/resource/Asturias> rdfs:label "Asturias"@es
        }
        "#;
        let result = pull_data_with_sparql( query ).await;
        assert!( result.to_string().contains( "false" ) );

    }
}
