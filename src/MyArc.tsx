
import React, { Component } from 'react';
import * as d3 from 'd3';

type MyProps = {
    width    : number,
    height   : number,
    thickness: number,
    progress : number,
};

type MyState = {
};

export default class MyArc extends Component<MyProps, MyState>
{
    static defaultProps = {
        width     : 300,
        height    : 300,
        thickness : 10,
        progress  : 0.1,
    };

    constructor(props)
    {
        super(props);
    }

    render()
    {
        return (
        <svg
            viewBox = {"0 0 "+this.props?.width+" "+this.props?.height}
            width   = {this.props?.width }
            height  = {this.props?.height}
            ref     = "root"
        >
        <g ref="group_main"></g>
        </svg>
        );
    }

    draw()
    {
        d3
        .select(this.refs.group_main)
        .selectAll("*")
        .remove();
    
        let arc_generator = d3.arc()
        .innerRadius((this.props.height/2) - this.props.thickness)
        .outerRadius((this.props.height/2));

        let arc_data = [{
            "startAngle": Math.PI * 0,
            "endAngle"  : Math.PI * 2 * this.props.progress,
            "padAngle"  : 0,
        }];

        // @ts-ignore
        this.g = d3
            .select(this.refs.group_main)
            .append("g")
            .attr("transform", "translate("+this.props.width/2+", "+this.props.height/2+")");
    
        // @ts-ignore
        this.g.selectAll('path').data(arc_data).enter()
            .append('path')
            .style("fill", function(d, i) { return "#fff"; })

            // End the path with rounded corners
            .style("stroke-linejoin", "round")
            .style("stroke-linecap", "round")
            
            .attr('d', arc_generator);

    }

    componentDidMount()
    {
    	this.draw();
    }

    componentDidUpdate()
    {
        this.draw();
    }
}