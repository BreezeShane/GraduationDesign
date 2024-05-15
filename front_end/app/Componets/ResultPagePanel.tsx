import { Button, Pagination, PaginationProps, Typography } from "antd";
import axios from "axios";
import { useState, memo, useEffect } from "react";

const { Title, Paragraph, Text, Link } = Typography;

const ResultPagePanel: React.FC<{ result_table: Object }> = memo((props) => {
    const { result_table } = props;
    const [results, setResults] = useState({});
    const [total, setTotal] = useState(0);
    const [current, setCurrent] = useState(0);
    const [title, setTitle] = useState("");
    const [link_content, setLinkContent] = useState("");
    const [link, setLink] = useState<string | undefined>();

    const updatePanelbyIndex = (page_num: number, raw_table: Object) => {
        let idx = page_num - 1;
        let table = Object.entries(raw_table);
        if (table.length > 0){
            let related_image_name = (table[idx][1])[0]
            let specie_name = (table[idx][1])[1];
            setLink(`http://baidu.com/s?wd=${specie_name}&q6=baike.baidu.com`);
            setLinkContent(`${specie_name}`);
            setTitle(`${specie_name} - ${related_image_name}`);
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

                { link && <Paragraph>
                    Click <a href={link}>{link_content}</a> For More Detalis.
                </Paragraph>}
                
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