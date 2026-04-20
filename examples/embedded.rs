use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig, EmbeddedError};
use api::{VectorsConfig, VectorParams, Distance, PointStruct, PointId, SearchRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = EmbeddedConfig::builder()
        .storage_path("./qdrant_data")
        .search_threads(4)
        .optimizer_threads(2)
        .build();

    // 初始化嵌入库
    let qdrant = QdrantEmbedded::new(config)?;

    // 创建集合
    let vectors_config = VectorsConfig {
        config: VectorParams {
            size: 128,
            distance: Distance::Cosine,
            ..Default::default()
        },
        ..Default::default()
    };

    qdrant.create_collection("my_collection".to_string(), vectors_config).await?;

    // 插入向量
    let points = vec![
        PointStruct {
            id: PointId::Num(1),
            vector: vec![0.1; 128].into(),
            payload: None,
        },
        PointStruct {
            id: PointId::Num(2),
            vector: vec![0.2; 128].into(),
            payload: None,
        },
    ];

    qdrant.upsert_points("my_collection".to_string(), points).await?;

    // 搜索
    let search_request = SearchRequest {
        vector: vec![0.15; 128].into(),
        limit: 10,
        ..Default::default()
    };

    let results = qdrant.search_points("my_collection".to_string(), search_request).await?;

    println!("Search results: {:?}", results);

    Ok(())
}
