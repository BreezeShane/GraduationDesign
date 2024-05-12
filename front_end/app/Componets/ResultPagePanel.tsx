import { Pagination, PaginationProps, Typography } from "antd";
import { useState, memo, useEffect } from "react";

const { Title, Paragraph, Text, Link } = Typography;

const ResultPagePanel: React.FC<{ result_table: Object }> = memo((props) => {
    const { result_table } = props;
    const [results, setResults] = useState({});
    const [total, setTotal] = useState(0);
    const [current, setCurrent] = useState(0);
    const [title, setTitle] = useState("");
    const [content, setContent] = useState("");
    const [link, generateLink] = useState<string | undefined>();

    const updatePanelbyIndex = (page_num: number, raw_table: Object) => {
        let idx = page_num - 1;
        let table = Object.entries(raw_table);
        if (table.length > 0){
            let related_image_name = (table[idx][1])[0]
            let specie_name = (table[idx][1])[1];
            generateLink(`${specie_name}`)
            setTitle(`${specie_name} - ${related_image_name}`);
            let content_to_set = "";
            if (specie_name == "Dasineura sp") {
                content_to_set = "荔枝叶瘿蚊，为双翅目，瘿蚊科。分布于广西、广东、海南等省区；以幼虫生箍，受害的点痕逐"
            } else if ( specie_name == "odontothrips loti" ) {
                content_to_set = "牛角花齿蓟马是我国北方为害苜蓿的重要害虫之一,严重影响苜蓿的产量与质量。"
            } else if ( specie_name == "Erythroneura apicalis" ) {
                content_to_set = "葡萄二星叶蝉，为 半翅目， 叶蝉科。中国葡萄产区均有发生。寄主于 葡萄、苹果、梨、桃花卉。成虫和若虫在叶背面吸汁液，被害叶面呈现小白斑点。严重时叶色苍白，以致焦枯脱落。"
            }
            setContent(content_to_set); //the content from website searched for specie_name
        }
    };

    useEffect(() => {
        setResults(result_table);
        setTotal(Object.keys(result_table).length);
        setCurrent(0);
        updatePanelbyIndex(1, result_table);
    }, [result_table])

    const onChange: PaginationProps['onChange'] = (page) => {
        setCurrent(page);
        updatePanelbyIndex(page, results)
    };
    return (
        <div style={{ height: "80%" }}>

            <div style={{ height: "80%" }}>
                <Title>{title}</Title>

                <Paragraph>
                    Click <a href={link}>{link}</a> For More Detalis.
                </Paragraph>
                <Paragraph>{content}</Paragraph>
            </div>
            <br />
            <Pagination
                total={total}
                showQuickJumper
                showTotal={(total, range) => `${range[0]} of ${total} items`}
                defaultCurrent={0}
                defaultPageSize={1}
                current={current}
                onChange={onChange}
            />
        </div>
    );
})

export default ResultPagePanel;