use prettytable::{ Table, row, csv::Writer };
use serde_json::Value;

/// returns data from Wikidata acording to the query
async fn pull_data_with_sparql( query : &str ) -> Value 
{

  let api = mediawiki::api::Api::new( "https://www.wikidata.org/w/api.php" )
  .await
  .unwrap(); // Will determine the SPARQL API URL via site info data
  if let Ok( res ) = api.sparql_query( query ).await
  {
    res
  }
  else
  {
    "Invalid query".into()
  }

}

/// builds and prints a table that contains:
/// 1) country name
/// 2) number of cities with population more than 1 000 000
/// 3) country population
/// 4) country area
/// writes this information into the CSV file 
#[ tokio::main ]
async fn main() -> std::io::Result< () >
{

  let query = r#"
  SELECT ?countryLabel (COUNT(DISTINCT ?city) AS ?count) (MAX(?countryPop) AS ?totalPopulation) (MAX(?area) AS ?totalArea) WHERE {
    ?city wdt:P31/wdt:P279* wd:Q515.   # отримати всі міста
    ?city wdt:P1082 ?population.       # отримати населення міста
    ?city wdt:P17 ?country.            # отримати країну, до якої належить місто
    ?country wdt:P30 ?continent.       # отримати континент, до якого належить країна
    ?country wdt:P2046 ?area.          # отримати площу країни
    FILTER(?area > 0)                  # фільтрувати країни з невірною площею
    ?country wdt:P1082 ?countryPop.    # отримати населення країни
    ?country wdt:P31 wd:Q3624078.      # отримати тільки суверенні держави
    FILTER(?population >= 1000000)     # фільтрувати тільки великі міста
    SERVICE wikibase:label {           # отримати назву країни
      bd:serviceParam wikibase:language "en" .
      }
    }
    GROUP BY ?countryLabel
    ORDER BY DESC(?count)
  "#;

  let res = pull_data_with_sparql( query ).await;

  // Create the table
  let mut table = Table::new();
  let result = res.as_object().unwrap()
  [ "results" ].as_object().unwrap()
  [ "bindings" ].as_array().unwrap()
  .into_iter()
  .map( | x | 
  {

    let row = x.as_object().unwrap();
    let count = row[ "count" ].as_object().unwrap()[ "value" ].as_str().unwrap();
    let country_label = row[ "countryLabel" ].as_object().unwrap()[ "value" ].as_str().unwrap();
    let total_area = row[ "totalArea" ].as_object().unwrap()[ "value" ].as_str().unwrap();
    let total_population = row[ "totalPopulation" ].as_object().unwrap()[ "value" ].as_str().unwrap().parse::<f64>().unwrap();

    ( country_label, count, total_area, total_population )

  }
  ).collect::< Vec< _ > >();

  // Add a row per time
  table.add_row( row![ "Country name", "Big cities number", "Population, mln", "Area, sq km" ] );
  let mut wtr = Writer::from_path( "result.csv" )?;

  for ( country_label, count, total_area, total_population ) in result 
  {
    table.add_row( row![ country_label, count, ( ( total_population / 1000.0 ) as i64 ).to_string(), total_area.to_string() ] );
    wtr.write_record( &[ country_label, count, &( ( total_population / 1000.0 ) as i64 ).to_string(), total_area ] )?;
  }

  wtr.flush()?;
  // Print the table to stdout
  table.printstd();
  Ok( () )
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

  #[ tokio::test ]
  async fn incorrect_query() 
  {

    let query = r#"
    qwe
    "#;
    let result = pull_data_with_sparql( query ).await;
    assert!( result.to_string().contains( "Invalid query" ) );

  }
}
